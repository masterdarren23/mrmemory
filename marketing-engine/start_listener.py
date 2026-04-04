"""Start the Telegram callback listener. Run as a background service."""
import os
import sys

os.chdir(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import config
config.load_env()

from telegram_bot import poll_updates
poll_updates()
