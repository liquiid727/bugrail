import { act, render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { NextIntlClientProvider } from "next-intl"
import { beforeEach, describe, expect, it, vi } from "vitest"
import enMessages from "@/i18n/messages/en.json"
import type { ContextOverview } from "@/lib/types"

const folderState = vi.hoisted(() => ({
  activeFolderId: 7 as number | null,
}))

const api = vi.hoisted(() => ({
  specosContextOverview: vi.fn(),
  specosContextPluginOperationsGet: vi.fn(),
  specosContextConfigSave: vi.fn(),
}))

vi.mock("@/contexts/active-folder-context", () => ({
  useActiveFolder: () => ({ activeFolderId: folderState.activeFolderId }),
}))
vi.mock("@/lib/api", () => api)
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}))

import { ContextPage } from "./context-page"

function overview(overrides?: Partial<ContextOverview>): ContextOverview {
  return {
    config: {
      version: 1,
      defaultLoadoutId: "default",
      providers: [
        {
          id: "local",
          kind: "local",
          endpoint: null,
          secretEnv: null,
          enabled: true,
          required: false,
          capabilities: [],
        },
      ],
      loadouts: [
        {
          id: "default",
          name: "Project essentials",
          sources: [{ path: "AGENTS.md", required: false, kind: "rules" }],
          providerIds: ["local"],
          maxItems: 16,
          maxBytes: 4096,
          maxTokens: 1000,
        },
      ],
    },
    providerHealth: [
      {
        id: "local",
        kind: "local",
        status: "healthy",
        message: null,
        checkedAt: "2026-08-13T00:00:00Z",
      },
    ],
    packages: [],
    activity: [],
    ...overrides,
  }
}

function renderPage() {
  return render(
    <NextIntlClientProvider locale="en" messages={enMessages}>
      <ContextPage />
    </NextIntlClientProvider>
  )
}

describe("ContextPage states (issues 053/055/058/060)", () => {
  beforeEach(() => {
    folderState.activeFolderId = 7
    vi.clearAllMocks()
    api.specosContextOverview.mockResolvedValue(overview())
    api.specosContextPluginOperationsGet.mockResolvedValue({
      config: [],
      validationErrors: [],
      health: [],
      jobs: [],
    })
  })

  it("shows no-workspace when no project is open", async () => {
    folderState.activeFolderId = null
    renderPage()
    expect(
      await screen.findByRole("heading", { name: "Open a project workspace" })
    ).toBeInTheDocument()
  })

  it("shows loading then overview empty packages", async () => {
    let resolveOverview: (value: ContextOverview) => void = () => {}
    api.specosContextOverview.mockReturnValue(
      new Promise<ContextOverview>((resolve) => {
        resolveOverview = resolve
      })
    )
    const { container } = renderPage()
    expect(container.querySelector("[aria-busy='true']")).not.toBeNull()
    resolveOverview(overview())
    expect(
      await screen.findByRole("heading", { name: "No Context Packages yet" })
    ).toBeInTheDocument()
  })

  it("shows healthy providers and editable loadout budgets", async () => {
    renderPage()
    await userEvent.click(await screen.findByRole("tab", { name: "Providers" }))
    expect(await screen.findAllByText("local")).not.toHaveLength(0)
    expect(screen.getByText("healthy")).toBeInTheDocument()
    await userEvent.click(screen.getByRole("tab", { name: "Loadouts" }))
    expect(screen.getByDisplayValue("16")).toBeInTheDocument()
    expect(
      screen.getByDisplayValue("AGENTS.md | rules | optional")
    ).toBeInTheDocument()
  })

  it("keeps the selected Context tab keyboard reachable", async () => {
    const user = userEvent.setup()
    renderPage()
    await screen.findByRole("heading", { name: "No Context Packages yet" })

    const tabList = screen.getByRole("tablist")
    const overviewTab = screen.getByRole("tab", { name: "Overview" })
    const codebaseTab = screen.getByRole("tab", { name: "Codebase" })
    expect(tabList).toHaveAttribute("tabindex", "0")

    act(() => tabList.focus())
    expect(overviewTab).toHaveFocus()
    await user.keyboard("{ArrowRight}")
    await waitFor(() => expect(codebaseTab).toHaveFocus())
  })

  it("shows degraded provider banner without replacing last-good data", async () => {
    api.specosContextOverview.mockResolvedValue(
      overview({
        providerHealth: [
          {
            id: "remote",
            kind: "tencent-memory",
            status: "degraded",
            message: "health endpoint returned 503",
            checkedAt: "2026-08-13T00:00:00Z",
          },
        ],
        config: {
          version: 1,
          defaultLoadoutId: "default",
          providers: [
            {
              id: "remote",
              kind: "tencent-memory",
              endpoint: "http://127.0.0.1:8125",
              secretEnv: "TENCENT_TOKEN",
              enabled: true,
              required: false,
              capabilities: ["memory"],
            },
          ],
          loadouts: overview().config.loadouts,
        },
      })
    )
    renderPage()
    expect(
      await screen.findByText(/optional provider\(s\) are degraded/)
    ).toBeInTheDocument()
    await userEvent.click(screen.getByRole("tab", { name: "Providers" }))
    expect(screen.getByText("degraded")).toBeInTheDocument()
    expect(screen.getByDisplayValue("TENCENT_TOKEN")).toBeInTheDocument()
    expect(screen.queryByDisplayValue(/sk-|Bearer/)).toBeNull()
  })

  it("joins package provenance onto overview and activity", async () => {
    api.specosContextOverview.mockResolvedValue(
      overview({
        packages: [
          {
            id: "ctx-1",
            taskId: 9,
            runSeq: 2,
            loadoutId: "default",
            status: "ready",
            contentHash: "abc",
            estimatedTokens: 12,
            totalBytes: 48,
            providerStatus: [],
            createdAt: "2026-08-13T00:00:00Z",
            items: [
              {
                id: "i1",
                ordinal: 0,
                kind: "rules",
                source: "AGENTS.md",
                title: "AGENTS.md",
                content: "# agents",
                contentHash: "def",
                required: true,
                provenance: { path: "AGENTS.md" },
              },
            ],
          },
        ],
        activity: [
          {
            id: 1,
            folderId: 7,
            packageId: "ctx-1",
            providerId: null,
            kind: "package",
            status: "ready",
            message: "1 items, 48 bytes",
            createdAt: "2026-08-13T00:00:00Z",
          },
        ],
      })
    )
    renderPage()
    expect(await screen.findByText("ctx-1")).toBeInTheDocument()
    expect(screen.getByText(/Task #9 · run 2/)).toBeInTheDocument()
    await userEvent.click(screen.getByRole("tab", { name: "Activity" }))
    expect(screen.getByText("package")).toBeInTheDocument()
    expect(screen.getByText("1 items, 48 bytes")).toBeInTheDocument()
  })

  it("shows persisted plugin health and job projections", async () => {
    api.specosContextPluginOperationsGet.mockResolvedValue({
      config: [
        {
          id: "wiki",
          kind: "wiki",
          adapter: "deterministic-wiki",
          enabled: true,
          required: false,
          capabilities: ["health", "search"],
          endpoint: null,
          secretEnvConfigured: false,
        },
      ],
      validationErrors: [],
      health: [
        {
          id: "wiki",
          kind: "wiki",
          status: "healthy",
          message: null,
          checkedAt: "2026-08-23T00:00:00Z",
        },
      ],
      jobs: [
        {
          id: 12,
          providerKind: "wiki",
          providerId: "wiki",
          operation: "sync",
          idempotencyKeyHash: "abcdef1234567890",
          requestHash: "0123456789abcdef",
          status: "queued",
          attemptCount: 0,
          maxAttempts: 3,
          nextRunAt: "2026-08-23T00:00:00Z",
          lastErrorCode: null,
          lastErrorMessage: null,
          createdAt: "2026-08-23T00:00:00Z",
          updatedAt: "2026-08-23T00:00:00Z",
          completedAt: null,
          attempts: [],
        },
      ],
    })
    renderPage()
    await userEvent.click(
      await screen.findByRole("tab", { name: "Operations" })
    )
    expect(await screen.findByText("wiki · sync")).toBeInTheDocument()
    expect(screen.getByText("queued")).toBeInTheDocument()
    expect(screen.getByText("wiki · health, search")).toBeInTheDocument()
    expect(screen.queryByText("0123456789abcdef")).toBeNull()
  })

  it("keeps last-good snapshot when refresh fails", async () => {
    renderPage()
    expect(
      await screen.findByRole("heading", { name: "Context" })
    ).toBeInTheDocument()
    api.specosContextOverview.mockRejectedValueOnce(new Error("socket closed"))
    await userEvent.click(screen.getByRole("button", { name: /Refresh/i }))
    await waitFor(() => {
      expect(
        screen.getByText(/Showing the last good snapshot/)
      ).toBeInTheDocument()
    })
    expect(screen.getByRole("heading", { name: "Context" })).toBeInTheDocument()
  })

  it("shows transport error with retry when nothing is loaded", async () => {
    api.specosContextOverview.mockRejectedValue(new Error("offline"))
    renderPage()
    expect(
      await screen.findByRole("heading", { name: "Could not load Context" })
    ).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument()
  })
})
