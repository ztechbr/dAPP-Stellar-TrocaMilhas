import os
import re
import sqlite3
import subprocess
from datetime import datetime, timezone
from functools import wraps
from pathlib import Path
from flask import Flask, render_template, request, redirect, flash, session, url_for, jsonify
from flask_wtf.csrf import CSRFProtect
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
from werkzeug.security import check_password_hash
from stellar_sdk import StrKey
from dotenv import load_dotenv

load_dotenv()
app = Flask(__name__)
app.config.update(SECRET_KEY=os.environ.get('FLASK_SECRET', ''), SESSION_COOKIE_HTTPONLY=True, SESSION_COOKIE_SAMESITE='Lax', SESSION_COOKIE_SECURE=os.getenv('COOKIE_SECURE', 'false').lower() == 'true', MAX_CONTENT_LENGTH=16384)
if len(app.config['SECRET_KEY']) < 32:
    raise RuntimeError('FLASK_SECRET must contain at least 32 characters')
if not os.environ.get('ADMIN_PASSWORD_HASH'):
    raise RuntimeError('Set ADMIN_PASSWORD_HASH with werkzeug.security.generate_password_hash')
csrf = CSRFProtect(app)
limiter = Limiter(get_remote_address, app=app, default_limits=['100 per hour'], storage_uri='memory://')
CONTRACT_ID = os.getenv('CONTRACT_ID', '')
NETWORK = os.getenv('NETWORK', 'testnet')
SOURCE_ALIAS = os.getenv('SOURCE_ALIAS', 'motoclube-admin')
DB = Path(os.getenv('AUDIT_DB', str(Path(__file__).parent / 'audit.sqlite3')))

def db():
    con = sqlite3.connect(DB)
    con.execute('CREATE TABLE IF NOT EXISTS allocations (id INTEGER PRIMARY KEY, timestamp TEXT NOT NULL, address TEXT NOT NULL, amount INTEGER NOT NULL, note TEXT, result TEXT NOT NULL)')
    con.commit()
    return con

def authenticated(view):
    @wraps(view)
    def wrapper(*args, **kwargs):
        if not session.get('admin'):
            return redirect(url_for('login'))
        return view(*args, **kwargs)
    return wrapper

def stellar_invoke(function, *args):
    if not CONTRACT_ID or not re.fullmatch(r'C[A-Z2-7]{55}', CONTRACT_ID):
        raise ValueError('Configure CONTRACT_ID válido')
    cmd = ['stellar', 'contract', 'invoke', '--id', CONTRACT_ID, '--source', SOURCE_ALIAS, '--network', NETWORK, '--', function, *args]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=120, check=True).stdout.strip()

@app.route('/login', methods=['GET', 'POST'])
@limiter.limit('8 per minute')
def login():
    if request.method == 'POST':
        if check_password_hash(os.environ['ADMIN_PASSWORD_HASH'], request.form.get('password', '')):
            session.clear()
            session['admin'] = True
            return redirect(url_for('index'))
        flash('Senha incorreta', 'error')
    return render_template('login.html')

@app.post('/logout')
@authenticated
def logout():
    session.clear()
    return redirect(url_for('login'))

@app.route('/', methods=['GET', 'POST'])
@authenticated
@limiter.limit('20 per minute', methods=['POST'])
def index():
    if request.method == 'POST':
        address = request.form.get('address', '').strip()
        note = request.form.get('note', '').strip()[:160]
        raw_amount = request.form.get('amount', '')
        try:
            amount = int(raw_amount)
            if not StrKey.is_valid_ed25519_public_key(address) or not (1 <= amount <= 1000000):
                raise ValueError('Endereço ou quantidade inválidos')
            result = stellar_invoke('allocate', '--to', address, '--amount', str(amount))
            with db() as con:
                con.execute('INSERT INTO allocations(timestamp,address,amount,note,result) VALUES(?,?,?,?,?)', (datetime.now(timezone.utc).isoformat(), address, amount, note, result[:300]))
            flash('Distribuição confirmada pela CLI: ' + result[:120], 'ok')
        except (ValueError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as exc:
            flash('Falha na distribuição: ' + str(exc)[:240], 'error')
        return redirect(url_for('index'))
    with db() as con:
        history = con.execute('SELECT timestamp,address,amount,note,result FROM allocations ORDER BY id DESC LIMIT 30').fetchall()
    return render_template('index.html', contract=CONTRACT_ID, history=history, network=NETWORK)

@app.get('/api/balance/<address>')
@limiter.limit('30 per minute')
def balance(address):
    if not StrKey.is_valid_ed25519_public_key(address):
        return jsonify(error='Endereço Stellar inválido'), 400
    try:
        return jsonify(address=address, balance=int(stellar_invoke('balance', '--user', address)), network=NETWORK)
    except (ValueError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return jsonify(error='Não foi possível consultar o contrato'), 503

@app.get('/api/status')
def status():
    return jsonify(network=NETWORK, configured=bool(CONTRACT_ID))

if __name__ == '__main__':
    app.run(host='127.0.0.1', port=5000, debug=False)
