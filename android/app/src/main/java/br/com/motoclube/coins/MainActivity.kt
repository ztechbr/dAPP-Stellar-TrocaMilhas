package br.com.motoclube.coins

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            var address by remember { mutableStateOf("") }
            var recipient by remember { mutableStateOf("") }
            var amount by remember { mutableStateOf("") }
            MaterialTheme(colorScheme = darkColorScheme(primary = Color(0xFFD4AF37))) {
                Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(24.dp), verticalArrangement = Arrangement.spacedBy(18.dp)) {
                    Text("◉  MOTOCLUBE COINS  ◉", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold, color = Color(0xFFD4AF37))
                    Text("Moedas de honra para quem compartilha a estrada")
                    ElevatedCard { Column(Modifier.padding(20.dp)) {
                        Text("SALDO STELLAR", color = Color(0xFFD4AF37))
                        Text("Conecte sua carteira", style = MaterialTheme.typography.headlineSmall)
                        Text("A consulta real requer integrar o endpoint RPC e o ID do contrato.")
                    } }
                    OutlinedTextField(address, {address=it}, label={Text("Seu endereço Stellar (G...)")}, modifier=Modifier.fillMaxWidth())
                    Button(onClick={}, enabled=false, modifier=Modifier.fillMaxWidth()) { Text("Consultar saldo, integração pendente") }
                    HorizontalDivider()
                    Text("Transferir moedas", style=MaterialTheme.typography.titleLarge)
                    OutlinedTextField(recipient, {recipient=it}, label={Text("Endereço do motociclista")}, modifier=Modifier.fillMaxWidth())
                    OutlinedTextField(amount, {amount=it}, label={Text("Quantidade")}, modifier=Modifier.fillMaxWidth())
                    Button(onClick={}, enabled=false, modifier=Modifier.fillMaxWidth()) { Text("Assinar e transferir, integração pendente") }
                    Text("Nenhuma chave privada é solicitada ou armazenada neste protótipo.", style=MaterialTheme.typography.bodySmall)
                }
            }
        }
    }
}
