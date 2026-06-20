#!/usr/bin/env python3
"""Capture screenshots of PrismNote frontend for documentation."""

import asyncio
import os
from pathlib import Path
from playwright.async_api import async_playwright

# Ensure screenshots directory exists
SCREENSHOTS_DIR = Path("docs/screenshots")
SCREENSHOTS_DIR.mkdir(parents=True, exist_ok=True)

async def capture_screenshots():
    """Capture various screenshots of the PrismNote application."""

    async with async_playwright() as p:
        # Launch browser
        browser = await p.chromium.launch()
        context = await browser.new_context(
            viewport={"width": 1400, "height": 900}
        )
        page = await context.new_page()

        # Navigate to the app
        print("Navigating to http://localhost:5173...")
        await page.goto("http://localhost:5173", wait_until="load")

        # Wait for app to render
        await page.wait_for_timeout(1000)

        # Screenshot 1: Welcome screen (initial state)
        print("Capturing welcome screen...")
        await page.screenshot(path=str(SCREENSHOTS_DIR / "01_welcome.png"))

        # Create a new notebook
        print("Creating new notebook...")
        create_btn = page.locator('button:has-text("New Notebook")')
        await create_btn.click()
        await page.wait_for_timeout(500)

        # Type notebook name
        input_field = page.locator('input[placeholder="Notebook name"]')
        await input_field.fill("Sample Analysis")

        # Click Create button
        create_confirm = page.locator('button:has-text("Create")').first
        await create_confirm.click()
        await page.wait_for_timeout(1500)

        # Screenshot 2: Notebook editor with dark theme
        print("Capturing notebook editor...")
        await page.screenshot(path=str(SCREENSHOTS_DIR / "02_notebook_dark.png"))

        # Screenshot 3: Light theme (toggle theme)
        print("Toggling to light mode...")
        theme_btn = page.locator('button:has-text("Light Mode")')
        if await theme_btn.count() > 0:
            await theme_btn.click()
            await page.wait_for_timeout(800)
            await page.screenshot(path=str(SCREENSHOTS_DIR / "03_notebook_light.png"))

        print(f"Screenshots saved to {SCREENSHOTS_DIR}")

        # Close browser
        await browser.close()

if __name__ == "__main__":
    asyncio.run(capture_screenshots())
