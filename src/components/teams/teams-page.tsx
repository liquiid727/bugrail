"use client"

import { useCallback, useEffect, useMemo, useState } from "react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import {
  Bot,
  CirclePause,
  CirclePlay,
  GitFork,
  Loader2,
  Network,
  Play,
  Plus,
  RefreshCw,
  Save,
  Square,
  ArrowUpRight,
} from "lucide-react"
import { useActiveFolder } from "@/contexts/active-folder-context"
import { useWorkbenchRoute } from "@/contexts/workbench-route-context"
import {
  specosAgentCatalogGet,
  specosAgentCatalogSave,
  specosTeamCatalogGet,
  specosTeamCatalogSave,
  specosTeamRunControl,
  specosTeamRunList,
  specosTeamRunStart,
} from "@/lib/api"
import { toErrorMessage } from "@/lib/app-error"
import type {
  AgentCatalog,
  AgentProfile,
  ModelProfile,
  TeamCatalog,
  TeamRunInfo,
  TeamWorkflowDefinition,
  WorkflowNodeDefinition,
} from "@/lib/types"
import { Checkbox } from "@/components/ui/checkbox"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Textarea } from "@/components/ui/textarea"

const EMPTY_AGENTS: AgentCatalog = {
  version: 1,
  defaultAgentProfileId: null,
  modelProfiles: [],
  agentProfiles: [],
}
const EMPTY_TEAMS: TeamCatalog = { version: 1, teams: [], workflows: [] }

function starterAgents(): AgentCatalog {
  return {
    version: 1,
    defaultAgentProfileId: "implementer",
    modelProfiles: [
      {
        id: "default-model",
        name: "Runtime default",
        providerRef: null,
        model: "",
        reasoning: null,
        fallbackProfileIds: [],
      },
    ],
    agentProfiles: ["planner", "implementer", "reviewer"].map((id) => ({
      id,
      name: id[0].toUpperCase() + id.slice(1),
      runtimeAdapter: "codex",
      modelProfileId: "default-model",
      modeId: null,
      reasoning: id === "reviewer" ? "high" : "medium",
      contextLoadoutId: "default",
      skills: [],
      rules: [],
      tools: [],
      configValues: {},
      enabled: true,
    })),
  }
}

function starterTeams(): TeamCatalog {
  return {
    version: 1,
    teams: [
      {
        id: "delivery-team",
        name: "Delivery team",
        description: "Plan, implement, and review in isolated WorkTasks.",
        memberProfileIds: ["planner", "implementer", "reviewer"],
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
            prompt:
              "Inspect the request and produce an implementation-ready plan.",
            agentProfileId: "planner",
            modelProfileId: null,
            contextLoadoutId: "default",
            dependsOn: [],
          },
          {
            id: "implement",
            title: "Implement",
            prompt:
              "Implement the approved plan and verify the changed behavior.",
            agentProfileId: "implementer",
            modelProfileId: null,
            contextLoadoutId: "default",
            dependsOn: ["plan"],
          },
          {
            id: "review",
            title: "Review",
            prompt:
              "Review the implementation against the task contract and report defects.",
            agentProfileId: "reviewer",
            modelProfileId: null,
            contextLoadoutId: "default",
            dependsOn: ["implement"],
          },
        ],
      },
    ],
  }
}

export function TeamsPageTitle() {
  const t = useTranslations("Teams")
  return <span className="text-sm font-medium">{t("title")}</span>
}

export function TeamsPage() {
  const t = useTranslations("Teams")
  const { activeFolderId } = useActiveFolder()
  const { openTaskDetail } = useWorkbenchRoute()
  const [agents, setAgents] = useState<AgentCatalog>(EMPTY_AGENTS)
  const [catalog, setCatalog] = useState<TeamCatalog>(EMPTY_TEAMS)
  const [runs, setRuns] = useState<TeamRunInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [busy, setBusy] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(
    async (background = false) => {
      if (activeFolderId == null) {
        setLoading(false)
        setAgents(EMPTY_AGENTS)
        setCatalog(EMPTY_TEAMS)
        setRuns([])
        return
      }
      if (background) {
        setRefreshing(true)
      } else {
        setLoading(true)
      }
      try {
        const [nextAgents, nextCatalog, nextRuns] = await Promise.all([
          specosAgentCatalogGet(activeFolderId),
          specosTeamCatalogGet(activeFolderId),
          specosTeamRunList(activeFolderId),
        ])
        setAgents(nextAgents)
        setCatalog(nextCatalog)
        setRuns(nextRuns)
        setError(null)
      } catch (cause) {
        setError(toErrorMessage(cause))
      } finally {
        setLoading(false)
        setRefreshing(false)
      }
    },
    [activeFolderId]
  )

  useEffect(() => {
    void load()
  }, [load])

  useEffect(() => {
    if (
      !runs.some((run) => !["done", "failed", "canceled"].includes(run.status))
    )
      return
    const timer = window.setInterval(() => void load(true), 2500)
    return () => window.clearInterval(timer)
  }, [load, runs])

  const saveProfiles = async () => {
    if (activeFolderId == null) return
    setBusy("profiles")
    try {
      setAgents(await specosAgentCatalogSave(activeFolderId, agents))
      toast.success(t("saved"))
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setBusy(null)
    }
  }

  const installStarter = async () => {
    if (activeFolderId == null) return
    setBusy("starter")
    try {
      const nextAgents = await specosAgentCatalogSave(
        activeFolderId,
        starterAgents()
      )
      const nextTeams = await specosTeamCatalogSave(
        activeFolderId,
        starterTeams()
      )
      setAgents(nextAgents)
      setCatalog(nextTeams)
      setError(null)
      toast.success(t("starterReady"))
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setBusy(null)
    }
  }

  const updateProfile = (id: string, patch: Partial<AgentProfile>) => {
    setAgents((current) => ({
      ...current,
      agentProfiles: current.agentProfiles.map((profile) =>
        profile.id === id ? { ...profile, ...patch } : profile
      ),
    }))
  }

  const updateModel = (id: string, patch: Partial<ModelProfile>) => {
    setAgents((current) => ({
      ...current,
      modelProfiles: current.modelProfiles.map((profile) =>
        profile.id === id ? { ...profile, ...patch } : profile
      ),
    }))
  }

  const updateWorkflow = (
    id: string,
    patch: Partial<TeamWorkflowDefinition>
  ) => {
    setCatalog((current) => ({
      ...current,
      workflows: current.workflows.map((workflow) =>
        workflow.id === id ? { ...workflow, ...patch } : workflow
      ),
    }))
  }

  const updateNode = (
    workflowId: string,
    nodeId: string,
    patch: Partial<WorkflowNodeDefinition>
  ) => {
    setCatalog((current) => ({
      ...current,
      workflows: current.workflows.map((workflow) =>
        workflow.id === workflowId
          ? {
              ...workflow,
              nodes: workflow.nodes.map((node) =>
                node.id === nodeId ? { ...node, ...patch } : node
              ),
            }
          : workflow
      ),
    }))
  }

  const saveTeams = async () => {
    if (activeFolderId == null) return
    setBusy("teams")
    try {
      setCatalog(await specosTeamCatalogSave(activeFolderId, catalog))
      toast.success(t("saved"))
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setBusy(null)
    }
  }

  const addProfile = () => {
    const n = agents.agentProfiles.length + 1
    setAgents((current) => ({
      ...current,
      agentProfiles: [
        ...current.agentProfiles,
        {
          id: `agent-${n}`,
          name: `Agent ${n}`,
          runtimeAdapter: "codex",
          modelProfileId: current.modelProfiles[0]?.id ?? null,
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
    }))
  }

  const runWorkflow = async (workflowId: string) => {
    if (activeFolderId == null) return
    setBusy(`run:${workflowId}`)
    try {
      await specosTeamRunStart(activeFolderId, workflowId)
      await load(true)
      toast.success(t("runStarted"))
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setBusy(null)
    }
  }

  const control = async (
    run: TeamRunInfo,
    action: "pause" | "resume" | "cancel"
  ) => {
    setBusy(`${action}:${run.id}`)
    try {
      await specosTeamRunControl(run.id, action)
      await load(true)
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setBusy(null)
    }
  }

  const hasConfiguration =
    agents.agentProfiles.length > 0 && catalog.workflows.length > 0
  const invalid = useMemo(
    () => [
      ...(agents.validationErrors ?? []),
      ...(catalog.validationErrors ?? []),
    ],
    [agents.validationErrors, catalog.validationErrors]
  )

  if (loading) {
    return (
      <div className="h-full space-y-4 overflow-auto p-6" aria-busy="true">
        <Skeleton className="h-24 w-full rounded-2xl" />
        <Skeleton className="h-64 w-full rounded-2xl" />
      </div>
    )
  }

  if (activeFolderId == null) {
    return (
      <CenteredState
        icon={Network}
        title={t("noWorkspace")}
        body={t("noWorkspaceHint")}
      />
    )
  }

  if (error && !hasConfiguration) {
    return (
      <CenteredState icon={Network} title={t("loadFailed")} body={error}>
        <Button onClick={() => void load()}>{t("retry")}</Button>
      </CenteredState>
    )
  }

  if (!hasConfiguration) {
    return (
      <CenteredState icon={Network} title={t("empty")} body={t("emptyHint")}>
        <Button onClick={() => void installStarter()} disabled={busy != null}>
          {busy === "starter" ? <Loader2 className="animate-spin" /> : <Plus />}
          {t("createStarter")}
        </Button>
      </CenteredState>
    )
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex items-center justify-between border-b px-6 py-3">
        <div>
          <h1 className="text-lg font-semibold">{t("title")}</h1>
          <p className="text-sm text-muted-foreground">{t("subtitle")}</p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => void load(true)}
          disabled={refreshing}
        >
          <RefreshCw className={refreshing ? "animate-spin" : ""} />
          {t("refresh")}
        </Button>
      </div>
      {error ? (
        <div className="border-b bg-destructive/10 px-6 py-2 text-sm text-destructive">
          {t("lastGood", { detail: error })}
        </div>
      ) : null}
      {invalid.length ? (
        <div className="border-b bg-amber-500/10 px-6 py-2 text-sm text-amber-700 dark:text-amber-300">
          {invalid.join(" · ")}
        </div>
      ) : null}
      <Tabs defaultValue="workflows" className="min-h-0 flex-1 p-4 md:p-6">
        <TabsList>
          <TabsTrigger value="workflows">{t("workflows")}</TabsTrigger>
          <TabsTrigger value="profiles">{t("profiles")}</TabsTrigger>
          <TabsTrigger value="runs">{t("runs")}</TabsTrigger>
        </TabsList>
        <TabsContent value="workflows" className="min-h-0 overflow-auto pt-3">
          <div className="mb-3 flex justify-end">
            <Button
              size="sm"
              onClick={() => void saveTeams()}
              disabled={busy != null}
            >
              {busy === "teams" ? (
                <Loader2 className="animate-spin" />
              ) : (
                <Save />
              )}
              {t("save")}
            </Button>
          </div>
          <div className="grid gap-4 xl:grid-cols-2">
            {catalog.workflows.map((workflow) => (
              <Card key={workflow.id} size="sm">
                <CardHeader>
                  <CardTitle>{workflow.name}</CardTitle>
                  <CardDescription>
                    {workflow.nodes.length} {t("nodes")} ·{" "}
                    {t("concurrency", { count: workflow.maxConcurrent })}
                  </CardDescription>
                  <CardAction>
                    <Button
                      size="sm"
                      onClick={() => void runWorkflow(workflow.id)}
                      disabled={busy != null || invalid.length > 0}
                    >
                      {busy === `run:${workflow.id}` ? (
                        <Loader2 className="animate-spin" />
                      ) : (
                        <Play />
                      )}
                      {t("run")}
                    </Button>
                  </CardAction>
                </CardHeader>
                <CardContent>
                  <label className="mb-3 grid gap-1 text-xs text-muted-foreground">
                    {t("maxConcurrent")}
                    <Input
                      type="number"
                      min={1}
                      max={32}
                      value={workflow.maxConcurrent}
                      onChange={(event) =>
                        updateWorkflow(workflow.id, {
                          maxConcurrent: Number(event.target.value),
                        })
                      }
                    />
                  </label>
                  <ol className="grid gap-2" aria-label={t("dagList")}>
                    {workflow.nodes.map((node) => (
                      <li
                        key={node.id}
                        className="grid gap-2 rounded-xl border bg-muted/20 p-3"
                      >
                        <div className="flex items-center gap-2">
                          <GitFork className="size-4 text-muted-foreground" />
                          <span className="font-medium">{node.title}</span>
                          <Badge variant="outline">{node.agentProfileId}</Badge>
                        </div>
                        <div className="mt-1 text-xs text-muted-foreground">
                          {node.dependsOn.length
                            ? `${t("after")}: ${node.dependsOn.join(", ")}`
                            : t("rootNode")}
                        </div>
                        <div className="grid gap-2 sm:grid-cols-2">
                          <label className="grid gap-1 text-xs text-muted-foreground">
                            {t("agentProfile")}
                            <Input
                              value={node.agentProfileId}
                              onChange={(event) =>
                                updateNode(workflow.id, node.id, {
                                  agentProfileId: event.target.value,
                                })
                              }
                            />
                          </label>
                          <label className="grid gap-1 text-xs text-muted-foreground">
                            {t("contextLoadout")}
                            <Input
                              value={node.contextLoadoutId ?? ""}
                              onChange={(event) =>
                                updateNode(workflow.id, node.id, {
                                  contextLoadoutId: event.target.value || null,
                                })
                              }
                            />
                          </label>
                          <label className="grid gap-1 text-xs text-muted-foreground sm:col-span-2">
                            {t("dependsOn")}
                            <Input
                              value={node.dependsOn.join(", ")}
                              onChange={(event) =>
                                updateNode(workflow.id, node.id, {
                                  dependsOn: csv(event.target.value),
                                })
                              }
                            />
                          </label>
                          <label className="grid gap-1 text-xs text-muted-foreground sm:col-span-2">
                            {t("prompt")}
                            <Textarea
                              className="min-h-20 text-xs"
                              value={node.prompt}
                              onChange={(event) =>
                                updateNode(workflow.id, node.id, {
                                  prompt: event.target.value,
                                })
                              }
                            />
                          </label>
                        </div>
                      </li>
                    ))}
                  </ol>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>
        <TabsContent value="profiles" className="min-h-0 overflow-auto pt-3">
          <div className="mb-3 flex justify-end gap-2">
            <Button variant="outline" size="sm" onClick={addProfile}>
              <Plus />
              {t("addProfile")}
            </Button>
            <Button
              size="sm"
              onClick={() => void saveProfiles()}
              disabled={busy != null}
            >
              {busy === "profiles" ? (
                <Loader2 className="animate-spin" />
              ) : (
                <Save />
              )}
              {t("save")}
            </Button>
          </div>
          <h3 className="mb-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {t("modelProfiles")}
          </h3>
          <div className="mb-5 grid gap-3 lg:grid-cols-2 xl:grid-cols-3">
            {agents.modelProfiles.map((profile) => (
              <Card key={profile.id} size="sm">
                <CardHeader>
                  <CardTitle>{profile.name}</CardTitle>
                  <CardDescription>{profile.id}</CardDescription>
                </CardHeader>
                <CardContent className="grid gap-3">
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("profileName")}
                    <Input
                      value={profile.name}
                      onChange={(event) =>
                        updateModel(profile.id, { name: event.target.value })
                      }
                    />
                  </label>
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("model")}
                    <Input
                      value={profile.model}
                      placeholder={t("runtimeDefault")}
                      onChange={(event) =>
                        updateModel(profile.id, { model: event.target.value })
                      }
                    />
                  </label>
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("reasoning")}
                    <Input
                      value={profile.reasoning ?? ""}
                      onChange={(event) =>
                        updateModel(profile.id, {
                          reasoning: event.target.value || null,
                        })
                      }
                    />
                  </label>
                </CardContent>
              </Card>
            ))}
          </div>
          <h3 className="mb-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {t("agentProfiles")}
          </h3>
          <div className="grid gap-3 lg:grid-cols-2 xl:grid-cols-3">
            {agents.agentProfiles.map((profile) => (
              <Card key={profile.id} size="sm">
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Bot className="size-4" />
                    {profile.name}
                  </CardTitle>
                  <CardDescription>{profile.id}</CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("profileName")}
                    <Input
                      value={profile.name}
                      onChange={(event) =>
                        updateProfile(profile.id, { name: event.target.value })
                      }
                    />
                  </label>
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("runtimeAdapter")}
                    <Input
                      value={profile.runtimeAdapter}
                      onChange={(event) =>
                        updateProfile(profile.id, {
                          runtimeAdapter: event.target.value,
                        })
                      }
                    />
                  </label>
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("modelProfile")}
                    <Input
                      value={profile.modelProfileId ?? ""}
                      onChange={(event) =>
                        updateProfile(profile.id, {
                          modelProfileId: event.target.value || null,
                        })
                      }
                    />
                  </label>
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("contextLoadout")}
                    <Input
                      value={profile.contextLoadoutId ?? ""}
                      onChange={(event) =>
                        updateProfile(profile.id, {
                          contextLoadoutId: event.target.value || null,
                        })
                      }
                    />
                  </label>
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("reasoning")}
                    <Input
                      value={profile.reasoning ?? ""}
                      onChange={(event) =>
                        updateProfile(profile.id, {
                          reasoning: event.target.value || null,
                        })
                      }
                    />
                  </label>
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("skills")}
                    <Input
                      value={profile.skills.join(", ")}
                      onChange={(event) =>
                        updateProfile(profile.id, {
                          skills: csv(event.target.value),
                        })
                      }
                    />
                  </label>
                  <label className="flex items-center gap-2 text-sm">
                    <Checkbox
                      checked={profile.enabled}
                      onCheckedChange={(value) =>
                        updateProfile(profile.id, { enabled: value === true })
                      }
                    />
                    {t("enabled")}
                  </label>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>
        <TabsContent value="runs" className="min-h-0 overflow-auto pt-3">
          {runs.length === 0 ? (
            <CenteredState
              icon={CirclePlay}
              title={t("noRuns")}
              body={t("noRunsHint")}
              compact
            />
          ) : (
            <div className="space-y-3">
              {runs.map((run) => (
                <Card key={run.id} size="sm">
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      {run.workflowId}
                      <StatusBadge status={run.status} />
                    </CardTitle>
                    <CardDescription>
                      {run.id} · v{run.workflowVersion}
                    </CardDescription>
                    <CardAction>
                      <div className="flex gap-1">
                        {run.controlState === "paused" ? (
                          <Button
                            variant="outline"
                            size="icon-sm"
                            title={t("resume")}
                            onClick={() => void control(run, "resume")}
                          >
                            <CirclePlay />
                          </Button>
                        ) : (
                          <Button
                            variant="outline"
                            size="icon-sm"
                            title={t("pause")}
                            onClick={() => void control(run, "pause")}
                          >
                            <CirclePause />
                          </Button>
                        )}
                        <Button
                          variant="outline"
                          size="icon-sm"
                          title={t("cancel")}
                          onClick={() => void control(run, "cancel")}
                          disabled={["done", "failed", "canceled"].includes(
                            run.status
                          )}
                        >
                          <Square />
                        </Button>
                      </div>
                    </CardAction>
                  </CardHeader>
                  <CardContent>
                    <div className="grid gap-2 md:grid-cols-3">
                      {run.nodes.map((node) => (
                        <button
                          type="button"
                          key={node.nodeId}
                          className="min-w-0 rounded-xl border p-3 text-left transition-colors hover:bg-muted/50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                          onClick={() => openTaskDetail(node.taskId)}
                          aria-label={`${t("openTask")}: ${node.title}`}
                        >
                          <div className="font-medium">{node.title}</div>
                          <div className="mt-1 flex items-center justify-between text-xs text-muted-foreground">
                            <span className="min-w-0 break-all">
                              #{node.taskId} · run {node.runSeq}
                            </span>
                            <span className="flex items-center gap-1">
                              <StatusBadge status={node.status} />
                              <ArrowUpRight
                                className="size-3.5"
                                aria-hidden="true"
                              />
                            </span>
                          </div>
                        </button>
                      ))}
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </TabsContent>
      </Tabs>
    </div>
  )
}

function StatusBadge({ status }: { status: string }) {
  const t = useTranslations("Teams")
  const variant =
    status === "failed" || status === "canceled"
      ? "destructive"
      : status === "done"
        ? "secondary"
        : "outline"
  const label = (() => {
    switch (status) {
      case "preparing":
        return t("status.preparing")
      case "running":
        return t("status.running")
      case "paused":
        return t("status.paused")
      case "review":
        return t("status.review")
      case "awaiting_input":
        return t("status.awaitingInput")
      case "merging":
        return t("status.merging")
      case "done":
        return t("status.done")
      case "failed":
        return t("status.failed")
      case "canceled":
        return t("status.canceled")
      default:
        return t("status.queued")
    }
  })()
  return <Badge variant={variant}>{label}</Badge>
}

function CenteredState({
  icon: Icon,
  title,
  body,
  children,
  compact = false,
}: {
  icon: typeof Network
  title: string
  body: string
  children?: React.ReactNode
  compact?: boolean
}) {
  return (
    <div
      className={
        compact
          ? "grid min-h-48 place-items-center"
          : "grid h-full place-items-center p-6"
      }
    >
      <div className="max-w-md text-center">
        <div className="mx-auto mb-3 grid size-12 place-items-center rounded-2xl bg-primary/10 text-primary">
          <Icon />
        </div>
        <h2 className="text-lg font-semibold">{title}</h2>
        <p className="mt-1 text-sm text-muted-foreground">{body}</p>
        {children ? (
          <div className="mt-4 flex justify-center">{children}</div>
        ) : null}
      </div>
    </div>
  )
}

function csv(value: string) {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean)
}
