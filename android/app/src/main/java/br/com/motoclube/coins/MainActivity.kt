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

private suspend fun readBalance(address: String): Long = withContext(Dispatchers.IO) {
    require(Regex("G[A-Z2-7]{55}").matches(address)) { "Endereço G inválido" }
    val url = URL("${BuildConfig.API_BASE_URL.trimEnd('/')}/api/balance/$address")
    val connection = (url.openConnection() as HttpURLConnection).apply {
        requestMethod = "GET"
        connectTimeout = 10000
        readTimeout = 30000
    }
    try {
        if (connection.responseCode != 200) error("Servidor retornou HTTP ${connection.responseCode}")
        JSONObject(connection.inputStream.bufferedReader().use { it.readText() }).getLong("balance")
    } finally { connection.disconnect() }
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            val scope = rememberCoroutineScope()
            var address by remember { mutableStateOf("") }
            var recipient by remember { mutableStateOf("") }
            var amount by remember { mutableStateOf("") }
            var balance by remember { mutableStateOf<Long?>(null) }
            var error by remember { mutableStateOf("") }
            var loading by remember { mutableStateOf(false) }
            MaterialTheme(colorScheme = darkColorScheme(primary = Color(0xFFD4AF37))) {
                Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(24.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    Text("MOTOCLUBE COINS", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold, color = Color(0xFFD4AF37))
                    Image(painterResource(R.drawable.ancient_coins), contentDescription = "Moedas antigas douradas", modifier = Modifier.fillMaxWidth().height(180.dp))
                    Text("Recompensas para quem compartilha a estrada")
                    ElevatedCard { Column(Modifier.fillMaxWidth().padding(20.dp)) {
                        Text("MEU SALDO", color = Color(0xFFD4AF37))
                        Text(balance?.toString() ?: "Não consultado", style = MaterialTheme.typography.headlineLarge)
                        Text("Moedas na Stellar Testnet")
                    } }
                    OutlinedTextField(address, { address = it.trim().uppercase(); balance = null }, label = { Text("Seu endereço Stellar (G...)") }, modifier = Modifier.fillMaxWidth())
                    Button(onClick = {
                        loading = true
                        error = ""
                        scope.launch {
                            try { balance = readBalance(address) } catch (e: Exception) { error = e.message ?: "Falha na consulta" } finally { loading = false }
                        }
                    }, enabled = !loading && Regex("G[A-Z2-7]{55}").matches(address), modifier = Modifier.fillMaxWidth()) { Text(if (loading) "Consultando..." else "Consultar saldo on-chain") }
                    if (error.isNotBlank()) Text(error, color = MaterialTheme.colorScheme.error)
                    HorizontalDivider()
                    Text("Transferir moedas", style = MaterialTheme.typography.titleLarge)
                    OutlinedTextField(recipient, { recipient = it.trim().uppercase() }, label = { Text("Endereço do participante") }, modifier = Modifier.fillMaxWidth())
                    OutlinedTextField(amount, { amount = it }, label = { Text("Quantidade") }, modifier = Modifier.fillMaxWidth())
                    Button(onClick = {}, enabled = false, modifier = Modifier.fillMaxWidth()) { Text("Transferência exige carteira assinante") }
                    Text("O aplicativo consulta saldos reais via API de leitura. Não coleta chaves privadas. Assinatura e envio de transferências ainda dependem de integrar uma carteira Stellar compatível com Soroban.", style = MaterialTheme.typography.bodySmall)
                    Text("API: ${BuildConfig.API_BASE_URL}", style = MaterialTheme.typography.labelSmall)
                }
            }
        }
    }
}
