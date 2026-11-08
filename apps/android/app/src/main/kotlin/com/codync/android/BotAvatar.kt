package com.codync.android

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.codync.android.core.Bot
import com.codync.android.design.*

/** A bot's mascot, or a group's members; the drawn character when this build has no picture for the id. */
@Composable internal fun BotAvatar(bot: Bot, bots: List<Bot>, size: Dp = 44.dp) {
    val mascot = if (bot.customIcon == null && bot.kind != "group") mascotDrawable(bot.avatar) else null
    when {
        mascot != null -> Image(
            painter = painterResource(mascot),
            contentDescription = null,
            modifier = Modifier.size(size).clip(CircleShape),
            contentScale = ContentScale.Fit,
        )
        bot.kind == "group" && bot.customIcon == null -> GroupAvatar(bot.memberBots(bots).map { AvatarFace(it.avatarShape, it.avatarColor, it.status, it.customIcon?.dataUrl, it.customIcon?.tinted == true) }, size)
        else -> CharacterAvatar(bot.avatarShape, bot.avatarColor, size, bot.status, customImage = bot.customIcon?.dataUrl, customTinted = bot.customIcon?.tinted == true)
    }
}

/** The roster's avatar, with its status dot (iOS `AvatarWithStatus`): amber needs you, the accent dot is unread. */
@Composable internal fun AvatarWithStatus(bot: Bot, bots: List<Bot>, size: Dp = 44.dp) {
    Box(Modifier.size(size)) {
        BotAvatar(bot, bots, size)
        if (bot.status == "needsInput") Box(Modifier.size(size * .36f).align(if (bot.lead) Alignment.TopEnd else Alignment.BottomEnd).ring()
            .background(Palette.warning, CircleShape), contentAlignment = Alignment.Center) {
            Symbol("exclamationmark", size = size * .24f, tint = Color.White)
        } else if (bot.unread > 0) Box(Modifier.size(size * .28f).align(if (bot.lead) Alignment.TopEnd else Alignment.BottomEnd).ring().background(Palette.accentFill, CircleShape))
        if (bot.lead) Box(Modifier.align(Alignment.BottomEnd)) { LeadAgentBadge(size * .34f) }
    }
}

/** A 2 dp ring in the background color around the dot, centered on its edge as iOS strokes it. */
@Composable private fun Modifier.ring(): Modifier {
    val color = Palette.background
    return drawWithContent {
        drawContent()
        drawCircle(color, size.minDimension / 2, style = Stroke(2.dp.toPx()))
    }
}
