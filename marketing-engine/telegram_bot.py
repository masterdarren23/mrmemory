"""Telegram bot that handles inline button callbacks for post approval."""

import json
import time
import threading
import requests

import config
config.load_env()

PENDING_POSTS = {}  # callback_id -> {text, platform, source_path}
_counter = 0


def send_post_for_approval(post_text: str, platform: str = "X", source_path: str = None):
    """Send a post to Telegram with Post Now / Skip buttons + full copyable text."""
    global _counter
    _counter += 1
    callback_id = f"post_{_counter}_{int(time.time())}"
    PENDING_POSTS[callback_id] = {"text": post_text, "platform": platform, "source_path": source_path}

    # Send the copyable post text first (plain, easy to copy)
    requests.post(
        f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/sendMessage",
        json={
            "chat_id": config.TELEGRAM_CHAT_ID,
            "text": f"📋 Draft for {platform}:\n\n{post_text}",
        },
        timeout=15,
    )
    
    time.sleep(0.5)

    # Then send the action buttons
    keyboard = {
        "inline_keyboard": [
            [
                {"text": "✅ Post Now", "callback_data": f"approve:{callback_id}"},
                {"text": "❌ Skip", "callback_data": f"skip:{callback_id}"},
            ]
        ]
    }

    requests.post(
        f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/sendMessage",
        json={
            "chat_id": config.TELEGRAM_CHAT_ID,
            "text": f"Post this to {platform}?",
            "reply_markup": keyboard,
        },
        timeout=15,
    )


def handle_callback(callback_query):
    """Process an inline button press."""
    data = callback_query.get("data", "")
    callback_id = callback_query.get("id")
    
    action, post_id = data.split(":", 1) if ":" in data else (data, "")

    if action == "approve" and post_id in PENDING_POSTS:
        post_data = PENDING_POSTS.pop(post_id)
        post_text = post_data["text"]
        platform = post_data.get("platform", "X")
        source_path = post_data.get("source_path")
        
        # Answer the callback
        requests.post(
            f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/answerCallbackQuery",
            json={"callback_query_id": callback_id, "text": f"Publishing..."},
            timeout=10,
        )

        if "blog" in platform.lower():
            # Publish to blog
            from blog_publisher import publish_and_deploy
            url = publish_and_deploy(source_path) if source_path else None
            if url:
                status = f"✅ Blog post published!\n{url}"
            else:
                status = "❌ Failed to publish blog post."
        else:
            # Post to X
            from x_poster import post_to_x
            success = post_to_x(post_text)
            status = "✅ Posted to X!" if success else "❌ Failed to post. Copy the text above and post manually."

        requests.post(
            f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/sendMessage",
            json={"chat_id": config.TELEGRAM_CHAT_ID, "text": status},
            timeout=15,
        )

    elif action == "skip":
        PENDING_POSTS.pop(post_id, None)
        requests.post(
            f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/answerCallbackQuery",
            json={"callback_query_id": callback_id, "text": "Skipped"},
            timeout=10,
        )

    else:
        requests.post(
            f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/answerCallbackQuery",
            json={"callback_query_id": callback_id, "text": "Post expired"},
            timeout=10,
        )


PENDING_QA = {}  # message_id -> {question, url, answer}


def send_qa_for_approval(question: str, url: str, answer: str):
    """Send a Q&A draft to Telegram. User replies 👍 to approve."""
    msg = f"💬 *Q&A Draft*\n\n*Q:* {question}\n*URL:* {url}\n\n*Draft Answer:*\n{answer[:800]}\n\n_Reply 👍 to post this answer_"
    resp = requests.post(
        f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/sendMessage",
        json={"chat_id": config.TELEGRAM_CHAT_ID, "text": msg, "parse_mode": "Markdown"},
        timeout=15,
    )
    if resp.ok:
        msg_id = str(resp.json().get("result", {}).get("message_id", ""))
        if msg_id:
            PENDING_QA[msg_id] = {"question": question, "url": url, "answer": answer}


def handle_thumbs_up(message):
    """Handle 👍 reply to a Q&A draft — copy answer to clipboard-ready message."""
    reply_to = message.get("reply_to_message", {})
    reply_id = str(reply_to.get("message_id", ""))
    
    if reply_id in PENDING_QA:
        qa = PENDING_QA.pop(reply_id)
        # Send the answer as a clean copyable message
        requests.post(
            f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/sendMessage",
            json={
                "chat_id": config.TELEGRAM_CHAT_ID,
                "text": f"✅ Approved! Copy and post:\n\n{qa['answer']}\n\n📎 {qa['url']}",
            },
            timeout=15,
        )
        return True
    return False


def poll_updates():
    """Long-poll Telegram for callback button presses and 👍 replies."""
    offset = 0
    print("[TELEGRAM BOT] Listening for button presses and 👍 approvals...")
    
    while True:
        try:
            resp = requests.get(
                f"https://api.telegram.org/bot{config.TELEGRAM_BOT_TOKEN}/getUpdates",
                params={"offset": offset, "timeout": 30, "allowed_updates": json.dumps(["callback_query", "message"])},
                timeout=35,
            )
            
            if resp.ok:
                for update in resp.json().get("result", []):
                    offset = update["update_id"] + 1
                    if "callback_query" in update:
                        handle_callback(update["callback_query"])
                    elif "message" in update:
                        msg = update["message"]
                        text = msg.get("text", "").strip()
                        # Check for 👍 reply to a Q&A draft
                        if text == "👍" and "reply_to_message" in msg:
                            handle_thumbs_up(msg)
        except Exception as e:
            print(f"[TELEGRAM BOT] Error: {e}")
            time.sleep(5)


def start_listener_thread():
    """Start the Telegram listener in a background thread."""
    t = threading.Thread(target=poll_updates, daemon=True)
    t.start()
    return t


if __name__ == "__main__":
    print("Starting Telegram bot listener...")
    poll_updates()
