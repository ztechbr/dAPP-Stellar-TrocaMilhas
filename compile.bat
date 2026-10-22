@echo off
REM ============================================================================
REM  compile.bat - compila cada parte do MotoClube Coins e faz o deploy do
REM  contrato na rede Stellar.
REM
REM  USO
REM    compile                 compila tudo (contrato + admin + android)
REM    compile contrato        testa e gera o .wasm do contrato Soroban
REM    compile admin           prepara o .venv e valida o painel Flask
REM    compile android         gera o APK de debug
REM    compile deploy          compila o contrato e publica na rede
REM    compile ajuda           mostra esta ajuda
REM
REM  VARIAVEIS OPCIONAIS (defina antes com: set NOME=valor)
REM    NETWORK        rede Stellar ............ padrao: testnet
REM    SOURCE_ALIAS   alias da chave na CLI ... padrao: motoclube-admin
REM    ADMIN_ADDRESS  administrador (G...) .... padrao: endereco do alias
REM    CAP            teto de moedas .......... padrao: 1000000
REM    API_BASE_URL   URL da API p/ o Android . padrao: http://10.0.2.2:5000
REM
REM  COMO LER ESTE ARQUIVO (tutorial)
REM    Cada parte do projeto e uma "sub-rotina" (:contrato, :admin, ...).
REM    O bloco PRINCIPAL so decide quais sub-rotinas chamar. Toda sub-rotina
REM    termina com "exit /b 0" (sucesso) ou salta para :falha, que devolve o
REM    codigo 1; assim o script para no primeiro erro, como um "set -e".
REM ============================================================================
setlocal EnableExtensions

set "ROOT=%~dp0"
if not defined NETWORK set "NETWORK=testnet"
if not defined SOURCE_ALIAS set "SOURCE_ALIAS=motoclube-admin"
if not defined CAP set "CAP=1000000"

set "WASM=%ROOT%contrato\target\wasm32v1-none\release\motoclube_coin.wasm"
set "STUDIO_JDK=%ProgramFiles%\Android\Android Studio\jbr"
set "SDK_DIR=%LOCALAPPDATA%\Android\Sdk"

REM ---------------------------------------------------------------- PRINCIPAL
set "ALVO=%~1"
if "%ALVO%"=="" set "ALVO=tudo"

if /i "%ALVO%"=="contrato" call :contrato  || goto :falha
if /i "%ALVO%"=="admin"    call :admin     || goto :falha
if /i "%ALVO%"=="android"  call :android   || goto :falha
if /i "%ALVO%"=="deploy"   call :deploy    || goto :falha
if /i "%ALVO%"=="tudo" (
    call :contrato || goto :falha
    call :admin    || goto :falha
    call :android  || goto :falha
)
if /i "%ALVO%"=="ajuda" goto :ajuda
for %%A in (contrato admin android deploy tudo) do if /i "%ALVO%"=="%%A" goto :sucesso
echo Alvo desconhecido: %ALVO%
call :ajuda
exit /b 1

:sucesso
echo.
echo [OK] "%ALVO%" concluido com sucesso.
exit /b 0

:falha
echo.
echo [ERRO] "%ALVO%" falhou. Veja as mensagens acima.
exit /b 1

:ajuda
echo.
echo Uso: compile [contrato ^| admin ^| android ^| deploy ^| tudo ^| ajuda]
echo.
echo   contrato   testa e gera o .wasm do contrato Soroban
echo   admin      prepara o .venv e valida o painel Flask
echo   android    gera o APK de debug
echo   deploy     compila o contrato e publica na rede (NETWORK=%NETWORK%)
echo   tudo       contrato + admin + android (padrao)
echo.
echo Variaveis opcionais: NETWORK, SOURCE_ALIAS, ADMIN_ADDRESS, CAP, API_BASE_URL
exit /b 0

REM ============================================================================
REM  :contrato - roda os testes e gera o WebAssembly do contrato Soroban.
REM  Pre-requisitos: Rust, alvo wasm32v1-none e Stellar CLI.
REM ============================================================================
:contrato
echo.
echo === [contrato] testes unitarios (cargo test) ===
call :exigir cargo   "Instale Rust em https://rustup.rs" || exit /b 1
call :exigir stellar "Instale a Stellar CLI: https://developers.stellar.org/docs/tools/cli" || exit /b 1
pushd "%ROOT%contrato"
rustup target add wasm32v1-none >nul 2>&1
cargo test || (popd & exit /b 1)

echo.
echo === [contrato] build do .wasm (stellar contract build) ===
stellar contract build || (popd & exit /b 1)
popd
echo WASM gerado em: %WASM%
exit /b 0

REM ============================================================================
REM  :deploy - compila e publica o contrato na rede escolhida.
REM
REM  Passo a passo:
REM    1. Garante que o alias de chave existe (na testnet, cria e financia).
REM    2. Descobre o ADMIN_ADDRESS a partir do alias, se nao foi informado.
REM    3. Compila (:contrato) e faz o deploy. O construtor do contrato recebe
REM       --admin e --cap na MESMA transacao, entao ninguem consegue se
REM       adiantar e tomar o controle do contrato recem-publicado.
REM    4. Salva o ID em contrato\contract_id.txt e atualiza admin\.env.
REM ============================================================================
:deploy
call :exigir stellar "Instale a Stellar CLI: https://developers.stellar.org/docs/tools/cli" || exit /b 1

stellar keys address %SOURCE_ALIAS% >nul 2>&1 || call :criar_chave || exit /b 1
if not defined ADMIN_ADDRESS (
    for /f "usebackq delims=" %%A in (`stellar keys address %SOURCE_ALIAS%`) do set "ADMIN_ADDRESS=%%A"
)

if /i "%NETWORK%"=="mainnet" call :confirmar_mainnet || exit /b 1

call :contrato || exit /b 1

echo.
echo === [deploy] publicando em %NETWORK% ===
echo   origem (assina e paga): %SOURCE_ALIAS%
echo   administrador:          %ADMIN_ADDRESS%
echo   teto de moedas:         %CAP%
stellar contract deploy --wasm "%WASM%" --source %SOURCE_ALIAS% --network %NETWORK% --alias motoclube_coin -- --admin %ADMIN_ADDRESS% --cap %CAP% > "%ROOT%contrato\contract_id.txt" || exit /b 1
set /p CONTRACT_ID=<"%ROOT%contrato\contract_id.txt"
echo.
echo CONTRACT_ID=%CONTRACT_ID%

call :atualizar_env
exit /b 0

:criar_chave
if /i not "%NETWORK%"=="testnet" (
    echo Alias "%SOURCE_ALIAS%" nao existe. Crie-o e financie-o antes do deploy:
    echo   stellar keys generate %SOURCE_ALIAS% --network %NETWORK%
    exit /b 1
)
echo Alias "%SOURCE_ALIAS%" nao existe: gerando e financiando na testnet...
stellar keys generate %SOURCE_ALIAS% --network testnet --fund
exit /b %errorlevel%

:confirmar_mainnet
echo ATENCAO: deploy na MAINNET usa dinheiro real e o contrato nao foi auditado.
set /p "RESPOSTA=Digite MAINNET para continuar: "
if "%RESPOSTA%"=="MAINNET" exit /b 0
echo Deploy cancelado.
exit /b 1

:atualizar_env
REM Grava o novo CONTRACT_ID no admin\.env (UTF-8 sem BOM), se o arquivo existir.
if not exist "%ROOT%admin\.env" (
    echo Dica: copie admin\.env.example para admin\.env e defina CONTRACT_ID=%CONTRACT_ID%
    exit /b 0
)
powershell -NoProfile -Command "$p='%ROOT%admin\.env'; $l=Get-Content $p | ForEach-Object { $_ -replace '^CONTRACT_ID=.*','CONTRACT_ID=%CONTRACT_ID%' }; [IO.File]::WriteAllLines($p,$l)"
echo admin\.env atualizado com o novo CONTRACT_ID.
exit /b 0

REM ============================================================================
REM  :admin - cria o ambiente virtual, instala dependencias e valida a sintaxe
REM  do painel Flask (compileall gera o bytecode de todos os .py).
REM ============================================================================
:admin
echo.
echo === [admin] painel Flask ===
call :exigir python "Instale Python 3.10+ em https://www.python.org" || exit /b 1
pushd "%ROOT%admin"
if not exist ".venv\Scripts\python.exe" (
    echo Criando ambiente virtual .venv...
    python -m venv .venv || (popd & exit /b 1)
)
".venv\Scripts\python.exe" -m pip install --quiet --disable-pip-version-check -r requirements.txt || (popd & exit /b 1)
".venv\Scripts\python.exe" -m compileall -q -x "\.venv" . || (popd & exit /b 1)
popd
echo Painel pronto. Para rodar: admin\.venv\Scripts\python admin\app.py
exit /b 0

REM ============================================================================
REM  :android - gera o APK de debug com o Gradle Wrapper do projeto.
REM  Usa o JDK embutido no Android Studio quando JAVA_HOME nao esta definido,
REM  e cria android\local.properties apontando para o Android SDK.
REM ============================================================================
:android
echo.
echo === [android] assembleDebug ===
if not defined JAVA_HOME if exist "%STUDIO_JDK%\bin\java.exe" set "JAVA_HOME=%STUDIO_JDK%"
if not defined JAVA_HOME (
    echo Defina JAVA_HOME com um JDK 17 ou 21 ^(o do Android Studio serve^).
    exit /b 1
)
if not exist "%ROOT%android\local.properties" if not defined ANDROID_HOME call :criar_local_properties || exit /b 1

set "GRADLE_ARGS="
if defined API_BASE_URL set "GRADLE_ARGS=-PAPI_BASE_URL=%API_BASE_URL%"
REM Caminho completo: o cmd pode estar configurado para nao buscar no diretorio atual.
call "%ROOT%android\gradlew.bat" -p "%ROOT%android" assembleDebug %GRADLE_ARGS% || exit /b 1
echo APK gerado em: %ROOT%android\app\build\outputs\apk\debug\app-debug.apk
exit /b 0

:criar_local_properties
if not exist "%SDK_DIR%\platforms" (
    echo Android SDK nao encontrado. Defina ANDROID_HOME ou instale pelo Android Studio.
    exit /b 1
)
REM O Gradle exige barras normais ou escapadas no caminho do SDK.
set "SDK_PROP=%SDK_DIR:\=/%"
> "%ROOT%android\local.properties" echo sdk.dir=%SDK_PROP%
echo Criado android\local.properties ^(sdk.dir=%SDK_PROP%^)
exit /b 0

REM ============================================================================
REM  :exigir COMANDO "DICA" - falha com uma dica se COMANDO nao estiver no PATH.
REM ============================================================================
:exigir
where %1 >nul 2>&1 && exit /b 0
echo Comando "%~1" nao encontrado. %~2
exit /b 1
