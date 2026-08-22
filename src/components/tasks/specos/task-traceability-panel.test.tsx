import { act, render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { NextIntlClientProvider } from "next-intl"
import { beforeEach, describe, expect, it, vi } from "vitest"
import enMessages from "@/i18n/messages/en.json"
import type { WorkTask } from "@/lib/types"

const api = vi.hoisted(() => ({
  specosContextPackageGet: vi.fn(),
  specosWorkTaskDependencies: vi.fn(),
  specosWorkTaskHandoffGet: vi.fn(),
  specosWorkTaskHandoffSave: vi.fn(),
  specosWorkTaskIntegrationPlan: vi.fn(),
  specosWorkTaskIntegrationRefresh: vi.fn(),
  specosWorkTaskRuns: vi.fn(),
  workTaskContractBind: vi.fn(),
  workTaskContractGet: vi.fn(),
  workTaskContractPreview: vi.fn(),
  workTaskGateDecision: vi.fn(),
  workTaskGateHumanDecide: vi.fn(),
  workTaskGateList: vi.fn(),
}))

vi.mock("@/lib/api", () => api)
vi.mock("sonner", () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}))

import { TaskTraceabilityPanel } from "./task-traceability-panel"

function task(taskKind?: WorkTask["task_kind"]): WorkTask {
  return {
    id: 11,
    folder_id: 1,
    title: "Verify handoff",
    task_kind: taskKind,
    config: {
      display_text: "Verify handoff",
      prompt_blocks: [],
      config_values: {},
    },
    status: "review",
    failure_reason: null,
    last_error: null,
    run_seq: 1,
    sort_order: 0,
    worktree_folder_id: null,
    conversation_id: null,
    connection_id: null,
    base_branch: null,
    base_sha: null,
    work_branch: null,
    cleanup_state: null,
    verdict: null,
    result_summary: null,
    files_changed: null,
    additions: null,
    deletions: null,
    merge_commit: null,
    preflight: null,
    archived_at: null,
    scheduled_at: null,
    created_at: "2026-08-13T00:00:00Z",
    updated_at: "2026-08-13T00:00:00Z",
    started_at: null,
    settled_at: null,
    finished_at: null,
  }
}

function renderPanel(taskKind?: WorkTask["task_kind"]) {
  return render(
    <NextIntlClientProvider locale="en" messages={enMessages}>
      <TaskTraceabilityPanel task={task(taskKind)} />
    </NextIntlClientProvider>
  )
}

describe("TaskTraceabilityPanel run / dependency / handoff / context", () => {
  beforeEach(() => {
    vi.clearAllMocks()
    api.workTaskContractGet.mockResolvedValue(null)
    api.workTaskGateDecision.mockResolvedValue(null)
    api.workTaskGateList.mockResolvedValue([])
    api.specosWorkTaskRuns.mockResolvedValue([])
    api.specosWorkTaskDependencies.mockResolvedValue([])
    api.specosWorkTaskHandoffGet.mockResolvedValue(null)
    api.specosWorkTaskIntegrationPlan.mockResolvedValue({
      taskId: 11,
      status: "no_sources",
      sources: [],
      conflicts: [],
    })
  })

  it("shows loading then empty run/context/handoff states", async () => {
    renderPanel()
    expect(screen.getByText("Loading persisted evidence…")).toBeInTheDocument()
    await userEvent.click(await screen.findByRole("tab", { name: /Runs/ }))
    expect(
      screen.getByText(/No persisted run snapshot yet/)
    ).toBeInTheDocument()
    await userEvent.click(screen.getByRole("tab", { name: /Context/ }))
    expect(
      screen.getByText(/No immutable Context Package is attached yet/)
    ).toBeInTheDocument()
    await userEvent.click(screen.getByRole("tab", { name: /Handoff/ }))
    expect(screen.getByPlaceholderText("Outcome summary")).toBeInTheDocument()
  })

  it("renders persisted run generations and blocking dependencies", async () => {
    api.specosWorkTaskRuns.mockResolvedValue([
      {
        taskId: 11,
        runSeq: 2,
        status: "review",
        agentProfileId: "implementer",
        modelProfileId: "shared",
        agentType: "codex",
        model: "gpt-test",
        modeId: null,
        reasoning: "medium",
        resolution: { reasonCodes: ["explicit_task_profile"] },
        conversationId: null,
        worktreeFolderId: null,
        contextPackageId: "ctx-9",
        createdAt: "2026-08-13T00:00:00Z",
        startedAt: null,
        finishedAt: null,
        updatedAt: "2026-08-13T00:00:00Z",
      },
    ])
    api.specosWorkTaskDependencies.mockResolvedValue([
      { parentTaskId: 3, childTaskId: 11, kind: "completion" },
    ])
    api.specosContextPackageGet.mockResolvedValue({
      id: "ctx-9",
      taskId: 11,
      runSeq: 2,
      loadoutId: "default",
      status: "ready",
      contentHash: "abc123def456",
      estimatedTokens: 20,
      totalBytes: 80,
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
          contentHash: "fff111aaa222",
          required: true,
          provenance: { path: "AGENTS.md" },
        },
      ],
    })
    renderPanel()
    await userEvent.click(await screen.findByRole("tab", { name: /Runs/ }))
    expect(screen.getByText("Run 2")).toBeInTheDocument()
    expect(screen.getByText(/implementer · gpt-test/)).toBeInTheDocument()
    expect(screen.getByText(/blocks this task/)).toBeInTheDocument()
    await userEvent.click(screen.getByRole("tab", { name: /Context/ }))
    expect(await screen.findByText("ctx-9")).toBeInTheDocument()
    expect(screen.getByText("AGENTS.md")).toBeInTheDocument()
    expect(screen.getByText("sha256:fff111aaa222")).toBeInTheDocument()
  })

  it("saves a handoff summary and shows transport error retry", async () => {
    api.specosWorkTaskHandoffSave.mockResolvedValue({
      taskId: 11,
      runSeq: 1,
      summary: "ready to integrate",
      artifacts: ["src/a.rs"],
      risks: [],
      openQuestions: [],
      createdAt: "2026-08-13T00:00:00Z",
    })
    renderPanel()
    await userEvent.click(await screen.findByRole("tab", { name: /Handoff/ }))
    await userEvent.type(
      screen.getByPlaceholderText("Outcome summary"),
      "ready to integrate"
    )
    await userEvent.type(
      screen.getByPlaceholderText("Artifacts (one path or reference per line)"),
      "src/a.rs"
    )
    await userEvent.click(screen.getByRole("button", { name: /Save handoff/ }))
    await waitFor(() => {
      expect(api.specosWorkTaskHandoffSave).toHaveBeenCalledWith(11, {
        summary: "ready to integrate",
        artifacts: ["src/a.rs"],
        risks: [],
        openQuestions: [],
      })
    })

    api.specosWorkTaskRuns.mockRejectedValueOnce(new Error("transport down"))
    await userEvent.click(
      screen.getByRole("button", { name: "Refresh evidence" })
    )
    expect(await screen.findByText("transport down")).toBeInTheDocument()
    expect(
      screen.getByRole("button", { name: /Retry: Spec traceability/ })
    ).toBeInTheDocument()
  })

  it("renders integration Git facts and refreshes a conflict plan through transport", async () => {
    api.specosWorkTaskIntegrationPlan.mockResolvedValue({
      taskId: 11,
      status: "conflict",
      conflicts: ["MERGE_HEAD"],
      sources: [
        {
          taskId: 3,
          title: "Source A",
          status: "review",
          runSeq: 2,
          branch: "source-a",
          currentHead: "abc123",
          capturedHead: "abc123",
          capturedRunSeq: 2,
          hasHandoff: true,
          handoffTrusted: true,
          gitBranchExists: true,
          specCurrent: true,
          gatesEligible: true,
          eligibilityReason: null,
          mergeOrder: 0,
          stale: false,
        },
      ],
    })
    api.specosWorkTaskIntegrationRefresh.mockResolvedValue({
      taskId: 11,
      status: "landed",
      conflicts: [],
      sources: [],
    })
    renderPanel("integration")

    await waitFor(() => {
      expect(api.specosWorkTaskIntegrationPlan).toHaveBeenCalledWith(11)
    })
    await userEvent.click(
      await screen.findByRole("tab", { name: /Integration/ })
    )
    expect(
      await screen.findByText("Merge recovery required: MERGE_HEAD")
    ).toBeInTheDocument()
    expect(screen.getByText("1. Source A")).toBeInTheDocument()
    expect(screen.getByText("abc123")).toBeInTheDocument()

    await userEvent.click(screen.getByRole("button", { name: /Refresh plan/ }))
    await waitFor(() => {
      expect(api.specosWorkTaskIntegrationRefresh).toHaveBeenCalledWith(11)
    })
    expect(screen.getByText("landed")).toBeInTheDocument()
  })

  it("previews and binds the exact selected Spec contract", async () => {
    const user = userEvent.setup()
    api.workTaskContractPreview.mockResolvedValue({
      source_spec_id: "BUGRAIL-SPECOS-001",
      source_spec_version: "0.3",
      source_spec_path:
        ".features/BUGRAIL-SPECOS-001-work-task-quality/spec.md",
      source_spec_hash: "approved-hash",
      acceptance_criteria: [
        { id: "BUGRAIL-SPECOS-001.AC01", title: "Bind", text: "Bind" },
        { id: "BUGRAIL-SPECOS-001.AC02", title: "Reject", text: "Reject" },
      ],
      current_binding_hash: null,
    })
    api.workTaskContractBind.mockResolvedValue(undefined)
    renderPanel()

    const path = await screen.findByLabelText("Feature Spec path")
    await user.clear(path)
    await user.type(
      path,
      ".features/BUGRAIL-SPECOS-001-work-task-quality/spec.md"
    )
    await user.click(screen.getByRole("button", { name: "Preview" }))
    await user.click(
      await screen.findByRole("checkbox", {
        name: /BUGRAIL-SPECOS-001\.AC02/,
      })
    )
    await user.click(
      screen.getByRole("checkbox", { name: "Require human approval" })
    )
    await user.click(
      await screen.findByRole("button", { name: "Bind 1 criteria" })
    )

    expect(api.workTaskContractBind).toHaveBeenCalledWith(11, {
      source_spec_path:
        ".features/BUGRAIL-SPECOS-001-work-task-quality/spec.md",
      expected_source_spec_hash: "approved-hash",
      selected_acceptance_criteria_ids: ["BUGRAIL-SPECOS-001.AC01"],
      gate_policy: {
        gates: [
          {
            id: "preflight",
            type: "preflight",
            required: true,
            reusable: true,
            allow_waiver: false,
          },
        ],
      },
    })
  })

  it("submits a reasoned human approval and keeps tabs keyboard reachable", async () => {
    const user = userEvent.setup()
    api.workTaskContractGet.mockResolvedValue({
      task_id: 11,
      source_spec_id: "BUGRAIL-SPECOS-001",
      source_spec_version: "0.3",
      source_spec_path:
        ".features/BUGRAIL-SPECOS-001-work-task-quality/spec.md",
      source_spec_hash: "approved-hash",
      acceptance_criteria: [
        { id: "BUGRAIL-SPECOS-001.AC01", title: "Bind", text: "Bind" },
      ],
      gate_policy: { gates: [] },
      created_at: "2026-08-23T00:00:00Z",
      updated_at: "2026-08-23T00:00:00Z",
    })
    api.workTaskGateDecision.mockResolvedValue({
      eligible: false,
      stale_spec: false,
      required: [
        {
          gate_id: "human-approval",
          gate_type: "human_approval",
          status: null,
          reason: "not run",
        },
      ],
      unmet: [
        {
          gate_id: "human-approval",
          gate_type: "human_approval",
          status: null,
          reason: "not run",
        },
      ],
      waived: [],
    })
    api.workTaskGateHumanDecide.mockResolvedValue(undefined)
    renderPanel()

    const contractTab = await screen.findByRole("tab", { name: /Contract/ })
    const runsTab = screen.getByRole("tab", { name: /Runs/ })
    const tabList = screen.getByRole("tablist")
    expect(tabList).toHaveAttribute("tabindex", "0")
    act(() => tabList.focus())
    expect(contractTab).toHaveFocus()
    await user.keyboard("{ArrowRight}")
    await waitFor(() => expect(runsTab).toHaveFocus())
    await user.keyboard("{ArrowLeft}")
    await waitFor(() => expect(contractTab).toHaveFocus())

    await user.type(screen.getByPlaceholderText("Decision reason"), "reviewed")
    await user.click(screen.getByRole("button", { name: "Approve" }))
    expect(api.workTaskGateHumanDecide).toHaveBeenCalledWith(
      11,
      "human-approval",
      "approve",
      "reviewed"
    )
  })
})
