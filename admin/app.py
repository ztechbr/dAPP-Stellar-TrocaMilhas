import os
import subprocess
from flask import Flask, render_template, request, redirect, flash
from stellar_sdk import StrKey

app = Flask(__name__)
app.secret_key = os.environ.get('FLASK_SECRET', 'change-me-in-production')
CONTRACT_ID = os.environ.get('CONTRACT_ID', '')
ADMIN_SECRET = os.environ.get('ADMIN_SECRET', '')

@app.route('/', methods=['GET', 'POST'])
def index():
    if request.method == 'POST':
        address = request.form.get('address', '').strip()
        try:
            amount = int(request.form.get('amount', '0'))
            if not StrKey.is_valid_ed25519_public_key(address) or amount <= 0:
                raise ValueError('Endereço ou quantidade inválida')
            if not CONTRACT_ID or not ADMIN_SECRET:
                raise ValueError('Configure CONTRACT_ID e ADMIN_SECRET')
            # Never pass the secret on the command line; use a configured local key alias.
            result = subprocess.run(['stellar', 'contract', 'invoke', '--id', CONTRACT_ID,
                '--source', 'motoclube-admin', '--network', 'testnet', '--',
                'allocate', '--to', address, '--amount', str(amount)],
                capture_output=True, text=True, timeout=90, check=True)
            flash('Distribuição submetida: ' + result.stdout[-200:], 'ok')
        except (ValueError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as exc:
            flash('Falha na operação: ' + str(exc)[:300], 'error')
        return redirect('/')
    return render_template('index.html', contract=CONTRACT_ID)

if __name__ == '__main__':
    app.run(host='127.0.0.1', port=5000, debug=False)
