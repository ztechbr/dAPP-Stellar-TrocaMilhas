/*
 * MotoClube Coins — aplicativo Android (Jetpack Compose)
 *
 * Tutorial: como esta tela funciona
 * ---------------------------------
 * 1. O usuário digita o próprio endereço Stellar (G...).
 * 2. Ao tocar em "Consultar saldo", [fetchBalance] chama a API Flask
 *    (`GET /api/balance/{endereço}`) numa thread de I/O.
 * 3. A API consulta o contrato na blockchain e devolve o saldo em JSON.
 * 4. O estado da tela ([BalanceUiState]) é atualizado e o Compose redesenha.
 *
 * O app NÃO guarda nem pede chaves privadas. A seção de transferência fica
 * desativada até integrar uma carteira capaz de assinar transações Soroban.
 */
package br.com.motoclube.coins

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.net.HttpURLConnection
import java.net.URL

/** Dourado da identidade visual do MotoClube. */
private val Gold = Color(0xFFD4AF37)

/** Conta Stellar pública: "G" + 55 caracteres base32 (A-Z, 2-7). */
private val STELLAR_ACCOUNT = Regex("G[A-Z2-7]{55}")

private const val CONNECT_TIMEOUT_MS = 10_000
private const val READ_TIMEOUT_MS = 30_000 // a API espera a resposta da blockchain

/** Verificação rápida de formato; o checksum completo é validado pela API. */
private fun isStellarAccount(address: String) = STELLAR_ACCOUNT.matches(address)

/**
 * Busca o saldo de [address] na API Flask.
 *
 * `withContext(Dispatchers.IO)` tira a chamada de rede da thread principal;
 * o Android encerraria o app se a interface ficasse travada esperando a rede.
 *
 * @throws IllegalArgumentException se o endereço tiver formato inválido.
 * @throws IllegalStateException se o servidor responder com erro.
 */
private suspend fun fetchBalance(address: String): Long = withContext(Dispatchers.IO) {
    require(isStellarAccount(address)) { "Endereço G inválido" }
    val url = URL("${BuildConfig.API_BASE_URL.trimEnd('/')}/api/balance/$address")
    val connection = (url.openConnection() as HttpURLConnection).apply {
        requestMethod = "GET"
        connectTimeout = CONNECT_TIMEOUT_MS
        readTimeout = READ_TIMEOUT_MS
    }
    try {
        check(connection.responseCode == HttpURLConnection.HTTP_OK) {
            "Servidor retornou HTTP ${connection.responseCode}"
        }
        val body = connection.inputStream.bufferedReader().use { it.readText() }
        JSONObject(body).getLong("balance")
    } finally {
        connection.disconnect()
    }
}

/**
 * Estado da consulta de saldo. Agrupar os campos num único objeto imutável
 * evita combinações impossíveis (ex.: carregando e com erro ao mesmo tempo).
 */
private data class BalanceUiState(
    val balance: Long? = null,
    val error: String = "",
    val loading: Boolean = false,
)

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme(colorScheme = darkColorScheme(primary = Gold)) {
                MotoClubeScreen()
            }
        }
    }
}

/** Tela principal: cabeçalho, consulta de saldo e transferência (desativada). */
@Composable
private fun MotoClubeScreen() {
    val scope = rememberCoroutineScope()
    var address by remember { mutableStateOf("") }
    var state by remember { mutableStateOf(BalanceUiState()) }

    fun queryBalance() {
        state = BalanceUiState(loading = true)
        scope.launch {
            state = try {
                BalanceUiState(balance = fetchBalance(address))
            } catch (e: Exception) {
                BalanceUiState(error = e.message ?: "Falha na consulta")
            }
        }
    }

    Column(
        Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Header()
        BalanceCard(state.balance)
        OutlinedTextField(
            value = address,
            onValueChange = { address = it.trim().uppercase(); state = BalanceUiState() },
            label = { Text("Seu endereço Stellar (G...)") },
            modifier = Modifier.fillMaxWidth(),
        )
        Button(
            onClick = ::queryBalance,
            enabled = !state.loading && isStellarAccount(address),
            modifier = Modifier.fillMaxWidth(),
        ) { Text(if (state.loading) "Consultando..." else "Consultar saldo on-chain") }
        if (state.error.isNotBlank()) Text(state.error, color = MaterialTheme.colorScheme.error)
        HorizontalDivider()
        TransferSection()
        Text("API: ${BuildConfig.API_BASE_URL}", style = MaterialTheme.typography.labelSmall)
    }
}

@Composable
private fun Header() {
    Text("MOTOCLUBE COINS", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold, color = Gold)
    Image(
        painterResource(R.drawable.ancient_coins),
        contentDescription = "Moedas antigas douradas",
        modifier = Modifier.fillMaxWidth().height(180.dp),
    )
    Text("Recompensas para quem compartilha a estrada")
}

@Composable
private fun BalanceCard(balance: Long?) {
    ElevatedCard {
        Column(Modifier.fillMaxWidth().padding(20.dp)) {
            Text("MEU SALDO", color = Gold)
            Text(balance?.toString() ?: "Não consultado", style = MaterialTheme.typography.headlineLarge)
            Text("Moedas na Stellar Testnet")
        }
    }
}

/**
 * Formulário de transferência mantido visível, mas desativado de propósito:
 * transferir exige que o titular assine com a própria chave, e o app não
 * deve manipular chaves privadas. Próximo passo: integrar uma carteira.
 */
@Composable
private fun TransferSection() {
    var recipient by remember { mutableStateOf("") }
    var amount by remember { mutableStateOf("") }
    Text("Transferir moedas", style = MaterialTheme.typography.titleLarge)
    OutlinedTextField(recipient, { recipient = it.trim().uppercase() }, label = { Text("Endereço do participante") }, modifier = Modifier.fillMaxWidth())
    OutlinedTextField(amount, { amount = it }, label = { Text("Quantidade") }, modifier = Modifier.fillMaxWidth())
    Button(onClick = {}, enabled = false, modifier = Modifier.fillMaxWidth()) { Text("Transferência exige carteira assinante") }
    Text(
        "O aplicativo consulta saldos reais via API de leitura. Não coleta chaves privadas. " +
            "Assinatura e envio de transferências ainda dependem de integrar uma carteira Stellar compatível com Soroban.",
        style = MaterialTheme.typography.bodySmall,
    )
}
