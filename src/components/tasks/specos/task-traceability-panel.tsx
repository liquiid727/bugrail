"use client"

import { useCallback, useEffect, useMemo, useState } from "react"
import { useTranslations } from "next-intl"
import {
  CheckCircle2,
  ClipboardCheck,
  FileCode2,
  History,
  Loader2,
  PackageSearch,
  RefreshCw,
  Send,
} from "lucide-react"
import { toast } from "sonner"

import {
  specosContextPackageGet,
  specosWorkTaskDependencies,
  specosWorkTaskHandoffGet,
  specosWorkTaskHandoffSave,
  specosWorkTaskRuns,
  workTaskContractBind,
  workTaskContractGet,
  workTaskContractPreview,
  workTaskGateDecision,
  workTaskGateHumanDecide,
  workTaskGateList,
} from "@/lib/api"
import { toErrorMessage } from "@/lib/app-error"
import type {
  ContextPackageInfo,
  WorkTask,
  WorkTaskContract,
  WorkTaskContractPreview,
  WorkTaskDependencyInfo,
  WorkTaskGateDecision,
  WorkTaskGateResult,
  WorkTaskHandoffInfo,
  WorkTaskRunInfo,
} from "@/lib/types"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Input } from "@/components/ui/input"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Textarea } from "@/components/ui/textarea"

interface TaskTraceabilityPanelProps {
  task: WorkTask
}

/**
 * SpecOS delivery evidence attached to one WorkTask. The panel deliberately
 * reads persisted projections instead of inferring run or gate state from the
 * live event stream, so reopening the task always reconstructs the same view.
 */
export function TaskTraceabilityPanel({ task }: TaskTraceabilityPanelProps) {
  const t = useTranslations("Tasks")
  const [contract, setContract] = useState<WorkTaskContract | null>(null)
  const [decision, setDecision] = useState<WorkTaskGateDecision | null>(null)
  const [gates, setGates] = useState<WorkTaskGateResult[]>([])
  const [runs, setRuns] = useState<WorkTaskRunInfo[]>([])
  const [dependencies, setDependencies] = useState<WorkTaskDependencyInfo[]>([])
  const [contextPackage, setContextPackage] =
    useState<ContextPackageInfo | null>(null)
  const [handoff, setHandoff] = useState<WorkTaskHandoffInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const reload = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const [
        nextContract,
        nextDecision,
        nextGates,
        nextRuns,
        nextDependencies,
        nextHandoff,
      ] = await Promise.all([
        workTaskContractGet(task.id),
        workTaskGateDecision(task.id).catch(() => null),
        workTaskGateList(task.id),
        specosWorkTaskRuns(task.id),
        specosWorkTaskDependencies(task.id),
        specosWorkTaskHandoffGet(task.id),
      ])
      setContract(nextContract)
      setDecision(nextDecision)
      setGates(nextGates)
      setRuns(nextRuns)
      setDependencies(nextDependencies)
      setHandoff(nextHandoff)

      const packageId = nextRuns.find(
        (run) => run.contextPackageId
      )?.contextPackageId
      setContextPackage(
        packageId ? await specosContextPackageGet(packageId) : null
      )
    } catch (cause) {
      setError(toErrorMessage(cause))
    } finally {
      setLoading(false)
    }
  }, [task.id])

  useEffect(() => {
    void reload()
  }, [reload, task.run_seq, task.status])

  return (
    <section className="flex flex-col gap-2">
      <div className="flex items-center justify-between gap-2">
        <h3 className="text-[0.6875rem] font-medium uppercase tracking-wide text-muted-foreground">
          {t("specosTitle")}
        </h3>
        <div className="flex items-center gap-1.5">
          <Badge variant={contract ? "secondary" : "outline"}>
            {contract ? t("specosBound") : t("specosUnbound")}
          </Badge>
          <Button
            type="button"
            size="icon-xs"
            variant="ghost"
            aria-label={t("specosRefresh")}
            disabled={loading}
            onClick={() => void reload()}
          >
            <RefreshCw className={loading ? "animate-spin" : undefined} />
          </Button>
        </div>
      </div>

      <div className="rounded-xl border border-border bg-muted/20 p-3">
        {loading ? (
          <div className="flex items-center gap-2 py-3 text-xs text-muted-foreground">
            <Loader2 className="size-3.5 animate-spin" aria-hidden="true" />
            {t("specosLoading")}
          </div>
        ) : error ? (
          <div className="flex flex-col items-start gap-2 py-2">
            <p className="text-xs text-destructive">{error}</p>
            <Button
              type="button"
              size="xs"
              variant="outline"
              aria-label={`${t("specosRetry")}: ${t("specosTitle")}`}
              onClick={() => void reload()}
            >
              {t("specosRetry")}
            </Button>
          </div>
        ) : (
          <Tabs defaultValue="contract">
            <TabsList
              variant="line"
              className="h-7 w-full justify-start overflow-x-auto"
            >
              <TabsTrigger
                value="contract"
                className="h-7 flex-none px-2 text-xs"
              >
                <FileCode2 /> {t("specosContractTab")}
              </TabsTrigger>
              <TabsTrigger value="runs" className="h-7 flex-none px-2 text-xs">
                <History /> {t("specosRunsTab")}
              </TabsTrigger>
              <TabsTrigger
                value="context"
                className="h-7 flex-none px-2 text-xs"
              >
                <PackageSearch /> {t("specosContextTab")}
              </TabsTrigger>
              <TabsTrigger
                value="handoff"
                className="h-7 flex-none px-2 text-xs"
              >
                <Send /> {t("specosHandoffTab")}
              </TabsTrigger>
            </TabsList>

            <TabsContent value="contract" className="pt-2">
              <ContractPane
                taskId={task.id}
                contract={contract}
                decision={decision}
                gates={gates}
                onChanged={reload}
              />
            </TabsContent>
            <TabsContent value="runs" className="pt-2">
              <RunsPane
                taskId={task.id}
                runs={runs}
                dependencies={dependencies}
              />
            </TabsContent>
            <TabsContent value="context" className="pt-2">
              <ContextPane packageInfo={contextPackage} />
            </TabsContent>
            <TabsContent value="handoff" className="pt-2">
              <HandoffPane
                taskId={task.id}
                handoff={handoff}
                onSaved={setHandoff}
              />
            </TabsContent>
          </Tabs>
        )}
      </div>
    </section>
  )
}

function ContractPane({
  taskId,
  contract,
  decision,
  gates,
  onChanged,
}: {
  taskId: number
  contract: WorkTaskContract | null
  decision: WorkTaskGateDecision | null
  gates: WorkTaskGateResult[]
  onChanged: () => Promise<void>
}) {
  const t = useTranslations("Tasks")
  const [path, setPath] = useState(contract?.source_spec_path ?? ".features/")
  const [preview, setPreview] = useState<WorkTaskContractPreview | null>(null)
  const [selected, setSelected] = useState<string[]>([])
  const [requirePreflight, setRequirePreflight] = useState(true)
  const [requireHuman, setRequireHuman] = useState(true)
  const [reason, setReason] = useState("")
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    if (contract) setPath(contract.source_spec_path)
  }, [contract])

  const inspect = async () => {
    setBusy(true)
    try {
      const next = await workTaskContractPreview(taskId, path.trim())
      setPreview(next)
      setSelected(next.acceptance_criteria.map((criterion) => criterion.id))
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setBusy(false)
    }
  }

  const bind = async () => {
    if (!preview) return
    setBusy(true)
    try {
      await workTaskContractBind(taskId, {
        source_spec_path: preview.source_spec_path,
        expected_source_spec_hash: preview.source_spec_hash,
        selected_acceptance_criteria_ids: selected,
        gate_policy: {
          gates: [
            ...(requirePreflight
              ? [
                  {
                    id: "preflight",
                    type: "preflight" as const,
                    required: true,
                    reusable: true,
                    allow_waiver: false,
                  },
                ]
              : []),
            ...(requireHuman
              ? [
                  {
                    id: "human-approval",
                    type: "human_approval" as const,
                    required: true,
                    reusable: false,
                    allow_waiver: true,
                  },
                ]
              : []),
          ],
        },
      })
      toast.success(t("specosContractSaved"))
      setPreview(null)
      await onChanged()
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setBusy(false)
    }
  }

  const decide = async (gateId: string, action: "approve" | "waive") => {
    if (!reason.trim()) {
      toast.error(t("specosReasonRequired"))
      return
    }
    setBusy(true)
    try {
      await workTaskGateHumanDecide(taskId, gateId, action, reason.trim())
      setReason("")
      toast.success(t("specosDecisionSaved"))
      await onChanged()
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setBusy(false)
    }
  }

  const humanGate = decision?.unmet.find(
    (gate) => gate.gate_type === "human_approval"
  )

  return (
    <div className="flex flex-col gap-3">
      {contract ? (
        <div className="rounded-lg border border-border bg-background/70 p-2.5 text-xs">
          <div className="flex items-start justify-between gap-2">
            <div className="min-w-0">
              <p className="font-medium">
                {contract.source_spec_id} · v{contract.source_spec_version}
              </p>
              <p
                className="truncate font-mono text-[0.625rem] text-muted-foreground"
                title={contract.source_spec_path}
              >
                {contract.source_spec_path}
              </p>
            </div>
            <Badge
              variant={
                decision?.eligible
                  ? "secondary"
                  : decision?.stale_spec
                    ? "destructive"
                    : "outline"
              }
            >
              {decision?.eligible
                ? t("specosEligible")
                : decision?.stale_spec
                  ? t("specosStale")
                  : t("specosGated")}
            </Badge>
          </div>
          <ul className="mt-2 flex flex-col gap-1 text-muted-foreground">
            {contract.acceptance_criteria.map((criterion) => (
              <li key={criterion.id} className="flex items-start gap-1.5">
                <CheckCircle2
                  className="mt-0.5 size-3 shrink-0"
                  aria-hidden="true"
                />
                <span>
                  <span className="font-mono text-[0.625rem]">
                    {criterion.id}
                  </span>{" "}
                  {criterion.title}
                </span>
              </li>
            ))}
          </ul>
        </div>
      ) : (
        <p className="text-xs text-muted-foreground">
          {t("specosContractEmpty")}
        </p>
      )}

      <div className="flex gap-2">
        <Input
          value={path}
          onChange={(event) => setPath(event.target.value)}
          className="h-8 font-mono text-xs"
          placeholder=".features/BUGRAIL-SPECOS-001-work-task-orchestration/feature-spec.md"
          aria-label={t("specosSpecPath")}
        />
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={busy || !path.trim()}
          onClick={() => void inspect()}
        >
          {contract ? t("specosRebind") : t("specosPreview")}
        </Button>
      </div>

      {preview ? (
        <div className="flex flex-col gap-2 rounded-lg border border-border bg-background/70 p-2.5">
          <p className="text-xs font-medium">
            {preview.source_spec_id} · v{preview.source_spec_version}
          </p>
          <div className="flex flex-col gap-1.5">
            {preview.acceptance_criteria.map((criterion) => (
              <label
                key={criterion.id}
                className="flex items-start gap-2 text-xs"
              >
                <Checkbox
                  checked={selected.includes(criterion.id)}
                  onCheckedChange={(checked) =>
                    setSelected((current) =>
                      checked === true
                        ? [...new Set([...current, criterion.id])]
                        : current.filter((id) => id !== criterion.id)
                    )
                  }
                />
                <span>
                  <span className="font-mono text-[0.625rem] text-muted-foreground">
                    {criterion.id}
                  </span>{" "}
                  {criterion.title}
                </span>
              </label>
            ))}
          </div>
          <div className="flex flex-wrap gap-x-4 gap-y-1 border-t border-border/70 pt-2">
            <label className="flex items-center gap-2 text-xs">
              <Checkbox
                checked={requirePreflight}
                onCheckedChange={(value) => setRequirePreflight(value === true)}
              />
              {t("specosPreflightGate")}
            </label>
            <label className="flex items-center gap-2 text-xs">
              <Checkbox
                checked={requireHuman}
                onCheckedChange={(value) => setRequireHuman(value === true)}
              />
              {t("specosHumanGate")}
            </label>
          </div>
          <Button
            type="button"
            size="sm"
            disabled={busy || selected.length === 0}
            onClick={() => void bind()}
          >
            <ClipboardCheck />{" "}
            {t("specosBindSelected", { count: selected.length })}
          </Button>
        </div>
      ) : null}

      {decision && contract ? (
        <div className="flex flex-col gap-1.5 border-t border-border/70 pt-2 text-xs">
          <div className="flex items-center justify-between gap-2">
            <span className="font-medium">{t("specosGateDecision")}</span>
            <span className="text-muted-foreground">
              {t("specosGateAttempts", { count: gates.length })}
            </span>
          </div>
          {decision.required.map((gate) => (
            <div
              key={gate.gate_id}
              className="flex items-start justify-between gap-2"
            >
              <span>{gate.gate_id}</span>
              <span
                className={
                  gate.status === "passed" || gate.status === "waived"
                    ? "text-emerald-600 dark:text-emerald-400"
                    : "text-muted-foreground"
                }
              >
                {gate.status ?? t("specosNotRun")} · {gate.reason}
              </span>
            </div>
          ))}
          {humanGate ? (
            <div className="mt-1 flex flex-col gap-2 rounded-lg bg-muted/60 p-2">
              <Input
                value={reason}
                onChange={(event) => setReason(event.target.value)}
                className="h-8 text-xs"
                placeholder={t("specosDecisionReason")}
              />
              <div className="flex gap-2">
                <Button
                  type="button"
                  size="xs"
                  disabled={busy}
                  onClick={() => void decide(humanGate.gate_id, "approve")}
                >
                  {t("specosApprove")}
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  disabled={busy}
                  onClick={() => void decide(humanGate.gate_id, "waive")}
                >
                  {t("specosWaive")}
                </Button>
              </div>
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  )
}

function RunsPane({
  taskId,
  runs,
  dependencies,
}: {
  taskId: number
  runs: WorkTaskRunInfo[]
  dependencies: WorkTaskDependencyInfo[]
}) {
  const t = useTranslations("Tasks")
  if (runs.length === 0 && dependencies.length === 0) {
    return (
      <p className="text-xs text-muted-foreground">{t("specosRunsEmpty")}</p>
    )
  }
  return (
    <div className="flex flex-col gap-3 text-xs">
      {dependencies.length > 0 ? (
        <div>
          <p className="mb-1 font-medium">{t("specosDependencies")}</p>
          <ul className="flex flex-col gap-1 text-muted-foreground">
            {dependencies.map((edge) => (
              <li
                key={`${edge.parentTaskId}:${edge.childTaskId}:${edge.kind}`}
                className="font-mono text-[0.6875rem]"
              >
                #{edge.parentTaskId} → #{edge.childTaskId} · {edge.kind}
                {edge.childTaskId === taskId
                  ? ` · ${t("specosBlocksThisTask")}`
                  : ""}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
      <ol className="flex flex-col gap-2">
        {runs.map((run) => (
          <li
            key={run.runSeq}
            className="rounded-lg border border-border bg-background/70 p-2.5"
          >
            <div className="flex items-center justify-between gap-2">
              <span className="font-medium">
                {t("specosRunNumber", { number: run.runSeq })}
              </span>
              <Badge variant={run.status === "done" ? "secondary" : "outline"}>
                {run.status}
              </Badge>
            </div>
            <p className="mt-1 text-muted-foreground">
              {[run.agentProfileId ?? run.agentType, run.model, run.modeId]
                .filter(Boolean)
                .join(" · ") || t("specosLegacyResolution")}
            </p>
            <p className="mt-1 font-mono text-[0.625rem] text-muted-foreground">
              {run.contextPackageId ?? t("specosNoContextPackage")}
            </p>
          </li>
        ))}
      </ol>
    </div>
  )
}

function ContextPane({
  packageInfo,
}: {
  packageInfo: ContextPackageInfo | null
}) {
  const t = useTranslations("Tasks")
  if (!packageInfo)
    return (
      <p className="text-xs text-muted-foreground">{t("specosContextEmpty")}</p>
    )
  return (
    <div className="flex flex-col gap-2 text-xs">
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0">
          <p className="font-medium">{packageInfo.id}</p>
          <p className="text-muted-foreground">
            {packageInfo.loadoutId} · ~{packageInfo.estimatedTokens} tokens ·{" "}
            {packageInfo.totalBytes} B
          </p>
        </div>
        <Badge
          variant={packageInfo.status === "ready" ? "secondary" : "outline"}
        >
          {packageInfo.status}
        </Badge>
      </div>
      <ul className="flex flex-col divide-y divide-border/70 overflow-hidden rounded-lg border border-border bg-background/70">
        {packageInfo.items.map((item) => (
          <li key={item.id} className="px-2.5 py-2">
            <div className="flex items-center justify-between gap-2">
              <span className="min-w-0 truncate font-mono text-[0.6875rem]">
                {item.source}
              </span>
              {item.required ? (
                <Badge variant="outline">{t("specosRequired")}</Badge>
              ) : null}
            </div>
            <p className="mt-0.5 truncate font-mono text-[0.625rem] text-muted-foreground">
              sha256:{item.contentHash.slice(0, 12)}
            </p>
          </li>
        ))}
      </ul>
    </div>
  )
}

function HandoffPane({
  taskId,
  handoff,
  onSaved,
}: {
  taskId: number
  handoff: WorkTaskHandoffInfo | null
  onSaved: (handoff: WorkTaskHandoffInfo) => void
}) {
  const t = useTranslations("Tasks")
  const [summary, setSummary] = useState(handoff?.summary ?? "")
  const [artifacts, setArtifacts] = useState(
    (handoff?.artifacts ?? []).join("\n")
  )
  const [risks, setRisks] = useState((handoff?.risks ?? []).join("\n"))
  const [questions, setQuestions] = useState(
    (handoff?.openQuestions ?? []).join("\n")
  )
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    setSummary(handoff?.summary ?? "")
    setArtifacts((handoff?.artifacts ?? []).join("\n"))
    setRisks((handoff?.risks ?? []).join("\n"))
    setQuestions((handoff?.openQuestions ?? []).join("\n"))
  }, [handoff])

  const lines = useMemo(
    () => (value: string) =>
      value
        .split("\n")
        .map((line) => line.trim())
        .filter(Boolean),
    []
  )
  const save = async () => {
    if (!summary.trim()) {
      toast.error(t("specosHandoffSummaryRequired"))
      return
    }
    setBusy(true)
    try {
      const saved = await specosWorkTaskHandoffSave(taskId, {
        summary: summary.trim(),
        artifacts: lines(artifacts),
        risks: lines(risks),
        openQuestions: lines(questions),
      })
      onSaved(saved)
      toast.success(t("specosHandoffSaved"))
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="flex flex-col gap-2">
      <Textarea
        value={summary}
        onChange={(event) => setSummary(event.target.value)}
        className="min-h-20 text-xs"
        placeholder={t("specosHandoffSummary")}
      />
      <Textarea
        value={artifacts}
        onChange={(event) => setArtifacts(event.target.value)}
        className="min-h-14 font-mono text-xs"
        placeholder={t("specosHandoffArtifacts")}
      />
      <Textarea
        value={risks}
        onChange={(event) => setRisks(event.target.value)}
        className="min-h-14 text-xs"
        placeholder={t("specosHandoffRisks")}
      />
      <Textarea
        value={questions}
        onChange={(event) => setQuestions(event.target.value)}
        className="min-h-14 text-xs"
        placeholder={t("specosHandoffQuestions")}
      />
      <Button
        type="button"
        size="sm"
        disabled={busy || !summary.trim()}
        onClick={() => void save()}
      >
        {busy ? <Loader2 className="animate-spin" /> : <Send />}{" "}
        {t("specosSaveHandoff")}
      </Button>
    </div>
  )
}
