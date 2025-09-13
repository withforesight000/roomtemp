package net.withforesight.roomtemp

import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import android.view.View
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    // 2) ルートビューにインセットを適用
    val content: View = findViewById(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(content) { v, insets ->
      val sys = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
      )
      // 上は前面のCSSで処理することも多いので 0 にしても良い。必要なら sys.top を使う。
      v.setPadding(sys.left, sys.top, sys.right, sys.bottom)
      insets
    }
  }
}
