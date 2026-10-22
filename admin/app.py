"""MotoClube Coins — painel administrativo e API de leitura.

Tutorial: como este módulo está organizado
==========================================

1. **Configuração** — lê o `.env`, valida segredos obrigatórios e prepara as
   proteções do Flask (cookies seguros, CSRF e limite de requisições).
2. **Infraestrutura** — duas pontes com o mundo externo:
   * `open_audit_db()`: SQLite local com o histórico de distribuições;
   * `invoke_contract()`: executa a Stellar CLI para falar com o contrato.
3. **Regras de entrada** — funções pequenas que validam endereço e quantidade
   antes de qualquer chamada à blockchain.
4. **Rotas** — login/logout, painel de distribuição e API pública somente
   leitura usada pelo aplicativo Android.

Por que usar a Stellar CLI e não uma chave no código?
-----------------------------------------------------
A chave secreta do administrador fica guardada no alias da CLI
(`stellar keys generate motoclube-admin`). Este processo nunca lê nem
transmite a chave: ele só pede para a CLI assinar. Assim o repositório, o
`.env` e os logs ficam livres de segredos de blockchain.
"""

import json
import os
import re
import sqlite3
import subprocess
from contextlib import closing
from datetime import datetime, timezone
from functools import wraps
from pathlib import Path

from dotenv import load_dotenv
from flask import Flask, flash, jsonify, redirect, render_template, request, session, url_for
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
from flask_wtf.csrf import CSRFProtect
from stellar_sdk import StrKey
from werkzeug.security import check_password_hash

# =============================================================================
# 1. Configuração
# =============================================================================
load_dotenv()

CONTRACT_ID = os.getenv('CONTRACT_ID', '')
NETWORK = os.getenv('NETWORK', 'testnet')
SOURCE_ALIAS = os.getenv('SOURCE_ALIAS', 'motoclube-admin')
AUDIT_DB_PATH = Path(os.getenv('AUDIT_DB', str(Path(__file__).parent / 'audit.sqlite3')))

MIN_SECRET_LENGTH = 32
MAX_ALLOCATION = 1_000_000
MAX_NOTE_LENGTH = 160
CLI_TIMEOUT_SECONDS = 120
CONTRACT_ID_PATTERN = re.compile(r'C[A-Z2-7]{55}')

# Erros esperados ao falar com a CLI; qualquer outro é bug e deve aparecer no log.
CLI_ERRORS = (ValueError, subprocess.CalledProcessError, subprocess.TimeoutExpired)

app = Flask(__name__)
app.config.update(
    SECRET_KEY=os.environ.get('FLASK_SECRET', ''),
    SESSION_COOKIE_HTTPONLY=True,  # JavaScript não lê o cookie de sessão
    SESSION_COOKIE_SAMESITE='Lax',  # cookie não viaja em POSTs de outros sites
    SESSION_COOKIE_SECURE=os.getenv('COOKIE_SECURE', 'false').lower() == 'true',
    MAX_CONTENT_LENGTH=16 * 1024,  # formulários são pequenos; corta payloads abusivos
)

# Falhar cedo: subir sem segredo forte deixaria sessões forjáveis.
if len(app.config['SECRET_KEY']) < MIN_SECRET_LENGTH:
    raise RuntimeError(f'FLASK_SECRET must contain at least {MIN_SECRET_LENGTH} characters')
if not os.environ.get('ADMIN_PASSWORD_HASH'):
    raise RuntimeError('Set ADMIN_PASSWORD_HASH with werkzeug.security.generate_password_hash')

csrf = CSRFProtect(app)
limiter = Limiter(get_remote_address, app=app, default_limits=['100 per hour'], storage_uri='memory://')


# =============================================================================
# 2. Infraestrutura
# =============================================================================
def open_audit_db():
    """Abre o SQLite de auditoria, criando a tabela na primeira execução.

    Use sempre com `closing(...)` para liberar o arquivo e com `with conn:`
    para confirmar (commit) a transação.
    """
    conn = sqlite3.connect(AUDIT_DB_PATH)
    conn.execute(
        'CREATE TABLE IF NOT EXISTS allocations ('
        'id INTEGER PRIMARY KEY, timestamp TEXT NOT NULL, address TEXT NOT NULL, '
        'amount INTEGER NOT NULL, note TEXT, result TEXT NOT NULL)'
    )
    conn.commit()
    return conn


def invoke_contract(function, *args):
    """Chama `function` no contrato via Stellar CLI e devolve a saída (stdout).

    Os argumentos vão em lista, nunca em string de shell, então não há risco
    de injeção de comandos. Lança `ValueError` se o contrato não estiver
    configurado e `CalledProcessError`/`TimeoutExpired` se a CLI falhar.
    """
    if not CONTRACT_ID_PATTERN.fullmatch(CONTRACT_ID):
        raise ValueError('Configure CONTRACT_ID válido')
    command = [
        'stellar', 'contract', 'invoke',
        '--id', CONTRACT_ID, '--source', SOURCE_ALIAS, '--network', NETWORK,
        '--', function, *args,
    ]
    completed = subprocess.run(command, capture_output=True, text=True, timeout=CLI_TIMEOUT_SECONDS, check=True)
    return completed.stdout.strip()


def parse_i128(cli_output):
    """Converte a saída da CLI para `int`.

    A Stellar CLI imprime valores `i128` como texto JSON entre aspas
    (ex.: `"50"`), pois eles não cabem num número JSON comum. Um `int()`
    direto falharia; por isso decodificamos o JSON primeiro.
    """
    return int(json.loads(cli_output))


# =============================================================================
# 3. Regras de entrada
# =============================================================================
def is_stellar_account(address):
    """True se `address` é uma conta Stellar pública válida (G..., com checksum)."""
    return StrKey.is_valid_ed25519_public_key(address)


def parse_allocation_amount(raw_amount):
    """Converte o texto do formulário em quantidade entre 1 e `MAX_ALLOCATION`."""
    amount = int(raw_amount)  # ValueError se não for número inteiro
    if not 1 <= amount <= MAX_ALLOCATION:
        raise ValueError('Quantidade fora do intervalo permitido')
    return amount


def login_required(view):
    """Decorador: redireciona para /login quem não está autenticado."""
    @wraps(view)
    def wrapper(*args, **kwargs):
        if not session.get('admin'):
            return redirect(url_for('login'))
        return view(*args, **kwargs)
    return wrapper


# =============================================================================
# 4. Rotas
# =============================================================================
@app.route('/login', methods=['GET', 'POST'])
@limiter.limit('8 per minute')  # freia tentativas de adivinhar a senha
def login():
    """Exibe o formulário e valida a senha contra o hash do `.env`."""
    if request.method == 'POST':
        if check_password_hash(os.environ['ADMIN_PASSWORD_HASH'], request.form.get('password', '')):
            session.clear()  # nova sessão evita fixação de sessão
            session['admin'] = True
            return redirect(url_for('index'))
        flash('Senha incorreta', 'error')
    return render_template('login.html')


@app.post('/logout')
@login_required
def logout():
    """Encerra a sessão. É POST (com CSRF) para que links externos não desloguem o admin."""
    session.clear()
    return redirect(url_for('login'))


@app.route('/', methods=['GET', 'POST'])
@login_required
@limiter.limit('20 per minute', methods=['POST'])
def index():
    """Painel: GET mostra o histórico; POST distribui moedas."""
    if request.method == 'POST':
        allocate_from_form()
        # Post/Redirect/Get: recarregar a página não reenvia a distribuição.
        return redirect(url_for('index'))
    with closing(open_audit_db()) as conn:
        history = conn.execute(
            'SELECT timestamp,address,amount,note,result FROM allocations ORDER BY id DESC LIMIT 30'
        ).fetchall()
    return render_template('index.html', contract=CONTRACT_ID, history=history, network=NETWORK)


def allocate_from_form():
    """Valida o formulário, chama `allocate` no contrato e registra o resultado.

    O histórico só grava distribuições que a CLI confirmou; falhas viram
    mensagem na tela e não poluem a auditoria.
    """
    address = request.form.get('address', '').strip()
    note = request.form.get('note', '').strip()[:MAX_NOTE_LENGTH]
    try:
        amount = parse_allocation_amount(request.form.get('amount', ''))
        if not is_stellar_account(address):
            raise ValueError('Endereço Stellar inválido')
        result = invoke_contract('allocate', '--to', address, '--amount', str(amount))
    except CLI_ERRORS as exc:
        flash('Falha na distribuição: ' + str(exc)[:240], 'error')
        return
    with closing(open_audit_db()) as conn, conn:
        conn.execute(
            'INSERT INTO allocations(timestamp,address,amount,note,result) VALUES(?,?,?,?,?)',
            (datetime.now(timezone.utc).isoformat(), address, amount, note, result[:300]),
        )
    flash('Distribuição confirmada pela CLI: ' + result[:120], 'ok')


@app.get('/api/balance/<address>')
@limiter.limit('30 per minute')
def balance(address):
    """API pública (somente leitura) usada pelo app Android.

    Resposta 200: `{"address": "G...", "balance": 50, "network": "testnet"}`.
    """
    if not is_stellar_account(address):
        return jsonify(error='Endereço Stellar inválido'), 400
    try:
        amount = parse_i128(invoke_contract('balance', '--user', address))
    except CLI_ERRORS:
        return jsonify(error='Não foi possível consultar o contrato'), 503
    return jsonify(address=address, balance=amount, network=NETWORK)


@app.get('/api/status')
def status():
    """Diagnóstico rápido: em qual rede estamos e se há contrato configurado."""
    return jsonify(network=NETWORK, configured=bool(CONTRACT_ID))


if __name__ == '__main__':
    # Servidor de desenvolvimento, preso ao localhost. Em produção use um
    # servidor WSGI (gunicorn/waitress) atrás de HTTPS.
    app.run(host='127.0.0.1', port=5000, debug=False)
