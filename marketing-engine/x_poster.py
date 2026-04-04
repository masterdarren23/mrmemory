"""X/Twitter poster using Playwright browser automation."""

import json
import random
import time
from pathlib import Path

from playwright.sync_api import sync_playwright

COOKIES_PATH = Path(__file__).parent / "data" / "x_cookies.json"
X_HANDLE = "realmrmemory"


def login_and_save_cookies():
    """Open your real Chrome browser for X login, then save cookies."""
    with sync_playwright() as p:
        # Use your real Chrome installation with a persistent profile
        # This avoids X's automation detection
        user_data_dir = str(Path.home() / ".marketing-engine-chrome")
        context = p.chromium.launch_persistent_context(
            user_data_dir,
            headless=False,
            channel="chrome",  # Use real Chrome, not Playwright's Chromium
            args=[
                "--disable-blink-features=AutomationControlled",
                "--no-first-run",
            ],
        )
        page = context.pages[0] if context.pages else context.new_page()
        page.goto("https://x.com/login")
        
        print("\n" + "="*50)
        print("Log in to X (@realmrmemory) in the Chrome window.")
        print("Once you see your feed, come back here and press Enter.")
        print("="*50 + "\n")
        input("Press Enter when logged in... ")
        
        # Save cookies
        cookies = context.cookies()
        COOKIES_PATH.parent.mkdir(parents=True, exist_ok=True)
        COOKIES_PATH.write_text(json.dumps(cookies), encoding="utf-8")
        print(f"Cookies saved to {COOKIES_PATH}")
        
        context.close()


def post_to_x(text: str) -> bool:
    """Post a tweet using saved cookies. Returns True on success."""
    if not COOKIES_PATH.exists():
        print("[X POSTER] No cookies found. Run login_and_save_cookies() first.")
        return False

    cookies = json.loads(COOKIES_PATH.read_text(encoding="utf-8"))

    try:
        with sync_playwright() as p:
            user_data_dir = str(Path.home() / ".marketing-engine-chrome")
            context = p.chromium.launch_persistent_context(
                user_data_dir,
                headless=True,
                channel="chrome",
                args=["--disable-blink-features=AutomationControlled"],
            )
            # Also load saved cookies as backup
            try:
                context.add_cookies(cookies)
            except Exception:
                pass

            page = context.pages[0] if context.pages else context.new_page()
            
            # Random delay to seem human
            time.sleep(random.uniform(2, 5))
            
            page.goto("https://x.com/compose/post", wait_until="networkidle", timeout=30000)
            time.sleep(random.uniform(2, 4))

            # Find the tweet compose box and type
            editor = page.locator('[data-testid="tweetTextarea_0"]')
            editor.click()
            time.sleep(random.uniform(0.5, 1.5))
            
            # Type slowly like a human
            for char in text:
                editor.type(char, delay=random.uniform(20, 80))
                if random.random() < 0.05:  # occasional pause
                    time.sleep(random.uniform(0.3, 1.0))

            time.sleep(random.uniform(1, 3))

            # Click the Post button
            post_button = page.locator('[data-testid="tweetButton"]')
            post_button.click()
            
            time.sleep(random.uniform(3, 5))

            # Update cookies (session refresh)
            new_cookies = context.cookies()
            COOKIES_PATH.write_text(json.dumps(new_cookies), encoding="utf-8")

            context.close()
            print(f"[X POSTER] Posted successfully: {text[:50]}...")
            return True

    except Exception as e:
        print(f"[X POSTER] Failed to post: {e}")
        return False


if __name__ == "__main__":
    import sys
    if len(sys.argv) > 1 and sys.argv[1] == "login":
        login_and_save_cookies()
    else:
        print("Usage:")
        print("  python x_poster.py login  — Open browser to log in and save cookies")
        print("  Import and use post_to_x('text') from other scripts")
