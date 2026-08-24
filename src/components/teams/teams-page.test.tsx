import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { NextIntlClientProvider } from "next-intl"
import { beforeEach, describe, expect, it, vi } from "vitest"
import enMessages from "@/i18n/messages/en.json"
import type { AgentCatalog, TeamCatalog } from "@/lib/types"
import {
  useWorkbenchRoute,
  WorkbenchRouteProvider,
} from "@/contexts/workbench-route-context"

const folderState = vi.hoisted(() => ({
  activeFolderId: 7 as number | null,
}))

const api = vi.hoisted(() => ({
  specosAgentCatalogGet: vi.fn(),
  specosAgentCatalogSave: vi.fn(),
  specosTeamCatalogGet: vi.fn(),
  specosTeamCatalogSave: vi.fn(),
  specosTeamRunList: vi.fn(),
  specosTeamRunControl: vi.fn(),
  specosTeamRunStart: vi.fn(),
}))

vi.mock("@/contexts/active-folder-context", () => ({
  useActiveFolder: () => ({ activeFolderId: folderState.activeFolderId }),
}))

vi.mock("@/lib/api", () => api)
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}))

import { TeamsPage } from "./teams-page"

const emptyAgents: AgentCatalog = {
  version: 1,
  defaultAgentProfileId: null,
  modelProfiles: [],
  agentProfiles: [],
}
const emptyTeams: TeamCatalog = { version: 1, teams: [], workflows: [] }

function configuredAgents(): AgentCatalog {
  return {
    version: 1,
    defaultAgentProfileId: "planner",
    modelProfiles: [
      {
        id: "shared",
        name: "Shared",
        providerRef: null,
        model: "gpt-test",
        reasoning: null,
        fallbackProfileIds: [],
      },
    ],
    agentProfiles: [
      {
        id: "planner",
        name: "Planner",
        runtimeAdapter: "codex",
        modelProfileId: "shared",
        modeId: null,
        reasoning: "medium",
        contextLoadoutId: "default",
        skills: [],
        rules: [],
        tools: [],
        configValues: {},
        enabled: true,
      },
    ],
  }
}

function configuredTeams(): TeamCatalog {
  return {
    version: 1,
    teams: [
      {
        id: "delivery-team",
        name: "Delivery team",
        description: "",
        memberProfileIds: ["planner"],
      },
    ],
    workflows: [
      {
        id: "delivery-workflow",
        name: "Delivery workflow",
        version: 1,
        teamId: "delivery-team",
        maxConcurrent: 2,
        nodes: [
          {
            id: "plan",
            title: "Plan",
            prompt: "Plan it",
            agentProfileId: "planner",
            modelProfileId: null,
            contextLoadoutId: "default",
            dependsOn: [],
          },
        ],
      },
    ],
  }
}

function renderPage() {
  return render(
    <NextIntlClientProvider locale="en" messages={enMessages}>
      <WorkbenchRouteProvider>
        <RouteProbe />
        <TeamsPage />
      </WorkbenchRouteProvider>
    </NextIntlClientProvider>
  )
}

function RouteProbe() {
  const { routeId, pendingTaskDetailId } = useWorkbenchRoute()
  return (
    <output data-testid="route-probe">
      {routeId}:{pendingTaskDetailId ?? "none"}
    </output>
  )
}

describe("TeamsPage / Agent Profile states (issue-045)", () => {
  beforeEach(() => {
    folderState.activeFolderId = 7
    vi.clearAllMocks()
    api.specosAgentCatalogGet.mockResolvedValue(emptyAgents)
    api.specosTeamCatalogGet.mockResolvedValue(emptyTeams)
    api.specosTeamRunList.mockResolvedValue([])
  })

  it("shows no-workspace when no project is open", async () => {
    folderState.activeFolderId = null
    renderPage()
    expect(
      await screen.findByRole("heading", { name: "Open a project workspace" })
    ).toBeInTheDocument()
  })

  it("shows loading then empty starter", async () => {
    let resolveAgents: (value: AgentCatalog) => void = () => {}
    api.specosAgentCatalogGet.mockReturnValue(
      new Promise<AgentCatalog>((resolve) => {
        resolveAgents = resolve
      })
    )
    const { container } = renderPage()
    expect(container.querySelector("[aria-busy='true']")).not.toBeNull()
    resolveAgents(emptyAgents)
    expect(
      await screen.findByRole("heading", { name: "No team configuration yet" })
    ).toBeInTheDocument()
    expect(
      screen.getByRole("button", { name: "Create starter team" })
    ).toBeInTheDocument()
  })

  it("saves starter profiles and shows the configured catalog", async () => {
    api.specosAgentCatalogSave.mockResolvedValue(configuredAgents())
    api.specosTeamCatalogSave.mockResolvedValue(configuredTeams())
    renderPage()
    await userEvent.click(
      await screen.findByRole("button", { name: "Create starter team" })
    )
    expect(await screen.findByText("Delivery workflow")).toBeInTheDocument()
    await userEvent.click(screen.getByRole("tab", { name: "Profiles" }))
    expect(await screen.findByText("Planner")).toBeInTheDocument()
    expect(screen.getByText("Shared")).toBeInTheDocument()
  })

  it("keeps last-good snapshot on refresh transport error", async () => {
    api.specosAgentCatalogGet.mockResolvedValueOnce(configuredAgents())
    api.specosTeamCatalogGet.mockResolvedValueOnce(configuredTeams())
    api.specosTeamRunList.mockResolvedValueOnce([])
    renderPage()
    expect(await screen.findByText("Delivery workflow")).toBeInTheDocument()
    api.specosAgentCatalogGet.mockRejectedValueOnce(new Error("transport down"))
    api.specosTeamCatalogGet.mockRejectedValueOnce(new Error("transport down"))
    api.specosTeamRunList.mockRejectedValueOnce(new Error("transport down"))
    await userEvent.click(screen.getByRole("button", { name: /Refresh/i }))
    await waitFor(() => {
      expect(
        screen.getByText(/Showing the last good snapshot/)
      ).toBeInTheDocument()
    })
    expect(screen.getByText("Delivery workflow")).toBeInTheDocument()
  })

  it("shows transport error with retry when nothing is loaded", async () => {
    api.specosAgentCatalogGet.mockRejectedValue(new Error("offline"))
    renderPage()
    expect(
      await screen.findByRole("heading", { name: "Could not load Agent Teams" })
    ).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument()
  })

  it("opens a run node in the regular task route", async () => {
    api.specosAgentCatalogGet.mockResolvedValue(configuredAgents())
    api.specosTeamCatalogGet.mockResolvedValue(configuredTeams())
    api.specosTeamRunList.mockResolvedValue([
      {
        id: "team-run-1",
        folderId: 7,
        teamId: "delivery-team",
        workflowId: "delivery-workflow",
        workflowVersion: 1,
        controlState: "running",
        status: "running",
        definitionHash: "hash",
        nodes: [
          {
            nodeId: "plan",
            taskId: 42,
            title: "Plan",
            status: "queued",
            runSeq: 1,
          },
        ],
        createdAt: "2026-08-23T00:00:00Z",
        updatedAt: "2026-08-23T00:00:00Z",
        finishedAt: null,
      },
    ])
    renderPage()

    await userEvent.click(await screen.findByRole("tab", { name: "Runs" }))
    await userEvent.click(
      screen.getByRole("button", { name: /Open task: Plan/ })
    )

    expect(screen.getByTestId("route-probe")).toHaveTextContent("tasks:42")
  })
})
