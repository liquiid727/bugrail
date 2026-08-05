import { render, screen } from "@testing-library/react"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

import { AppI18nProvider } from "./i18n-provider"
import enMessages from "@/i18n/messages/en.json"
import { getSystemLanguageSettings } from "@/lib/api"

vi.mock("@/lib/api", () => ({
  getSystemLanguageSettings: vi.fn(),
}))

describe("AppI18nProvider language settings fallback", () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  beforeEach(() => {
    vi.mocked(getSystemLanguageSettings).mockReset()
  })

  it("renders with the initial locale when backend settings are unavailable", async () => {
    vi.mocked(getSystemLanguageSettings).mockRejectedValue(
      new Error("backend unavailable")
    )
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => undefined)
    const consoleWarn = vi
      .spyOn(console, "warn")
      .mockImplementation(() => undefined)

    render(
      <AppI18nProvider initialLocale="en" initialMessages={enMessages}>
        <div>Bugrail ready</div>
      </AppI18nProvider>
    )

    expect(await screen.findByText("Bugrail ready")).toBeVisible()
    expect(consoleError).not.toHaveBeenCalled()
    expect(consoleWarn).toHaveBeenCalledWith(
      "[i18n] using initial language settings:",
      expect.any(Error)
    )
  })
})
