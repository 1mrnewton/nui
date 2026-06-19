// GENERATED from counter.nui by nuic — do not edit.
package dev.nui.generated

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Remove
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.nui.runtime.Bridge
import kotlinx.serialization.Serializable

@Serializable
data class CounterState(val count: Int = 0, val step: Int = 1, val show_label: Boolean = true, val title: String = "Counter")

class CounterModel(private val bridge: Bridge) {
    var state by mutableStateOf(CounterState())
        private set

    init { bridge.onState<CounterState> { new -> state = new } }

    fun increment() = bridge.send("increment")

    fun decrement() = bridge.send("decrement")

    fun reset() = bridge.send("reset")

    fun set_step(value: Int) = bridge.send("set_step", mapOf("value" to value))

    fun set_show_label(value: Boolean) = bridge.send("set_show_label", mapOf("value" to value))

    fun set_title(value: String) = bridge.send("set_title", mapOf("value" to value))

}

@Composable
fun CounterView(bridge: Bridge) {
    val model = remember { CounterModel(bridge) }
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
    ) {
        Column(
            modifier = Modifier.padding(24.dp).fillMaxWidth(),
            verticalArrangement = Arrangement.spacedBy(20.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Icon(
                    imageVector = Icons.Filled.Refresh,
                    contentDescription = "refresh",
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(22.dp)
                )
                Text(
                    text = "${model.state.title}",
                    fontSize = 28.sp,
                    fontWeight = FontWeight.Bold
                )
            }
            Card(
                modifier = Modifier.fillMaxWidth()
            ) {
                    Column(
                        modifier = Modifier.fillMaxWidth(),
                        verticalArrangement = Arrangement.spacedBy(16.dp),
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        Text(
                            text = "${model.state.count}",
                            fontSize = 72.sp,
                            fontWeight = FontWeight.Bold
                        )
                        if (model.state.show_label) {
                            Text(
                                text = "step by ${model.state.step}",
                                fontSize = 14.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        } else {
                            Text(
                                text = "subtitle hidden",
                                fontSize = 14.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        LinearProgressIndicator(
                            progress = { (model.state.count.toFloat() / 20f).coerceIn(0f, 1f) },
                            modifier = Modifier.fillMaxWidth()
                        )
                        Row(
                            horizontalArrangement = Arrangement.spacedBy(20.dp)
                        ) {
                            FilledIconButton(
                                onClick = model::decrement,
                                colors = IconButtonDefaults.filledIconButtonColors(containerColor = Color.Red)
                            ) { Icon(Icons.Filled.Remove, contentDescription = "minus") }
                            FilledIconButton(
                                onClick = model::increment,
                                colors = IconButtonDefaults.filledIconButtonColors(containerColor = Color.Green)
                            ) { Icon(Icons.Filled.Add, contentDescription = "plus") }
                        }
                        OutlinedButton(onClick = model::reset) { Text("Reset") }
                    }
            }
            HorizontalDivider()
            Column(
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Column(
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text("Step", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Slider(
                        value = model.state.step.toFloat(),
                        onValueChange = { model.set_step(it.toInt()) },
                        valueRange = 1f..10f,
                        steps = (10 - 1 - 1).coerceAtLeast(0)
                    )
                }
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("Show subtitle")
                    Switch(
                        checked = model.state.show_label,
                        onCheckedChange = model::set_show_label
                    )
                }
                OutlinedTextField(
                    value = model.state.title,
                    onValueChange = model::set_title,
                    label = { Text("Title") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(capitalization = KeyboardCapitalization.Words)
                )
            }
        }
    }
}
