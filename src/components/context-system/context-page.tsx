"use client"

import { useCallback, useEffect, useState } from "react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import {
  BrainCircuit,
  Database,
  Loader2,
  Play,
  RefreshCw,
  Save,
  ShieldAlert,
  Waypoints,
} from "lucide-react"
import { useActiveFolder } from "@/contexts/active-folder-context"
import {
  codeIntelligenceGetState,
  codeIntelligenceInstall,
  codeIntelligenceOpenGraph,
  codeIntelligenceReindex,
  codeIntelligenceSetBinaryOverride,
  codeIntelligenceSetEnabled,
  specosContextConfigSave,
  specosContextOverview,
  specosMemoryDeliveryList,
  specosMemoryDeliveryRetry,
  specosMemoryProviderTest,
  specosMemoryRecallPreview,
} from "@/lib/api"
import { toErrorMessage } from "@/lib/app-error"
import { isDesktop, isRemoteDesktopMode } from "@/lib/transport"
import type {
  CodeIntelState,
  ContextLoadout,
  ContextOverview,
  ContextProviderConfig,
  MemoryDeliveryInfo,
  MemoryProviderTestResult,
  MemoryRecallPreview,
} from "@/lib/types"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Checkbox } from "@/components/ui/checkbox"
import { Skeleton } from "@/components/ui/skeleton"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Textarea } from "@/components/ui/textarea"

export function ContextPageTitle() {
  const t = useTranslations("ContextSystem")
  return <span className="text-sm font-medium">{t("title")}</span>
}

export function ContextPage() {
  const t = useTranslations("ContextSystem")
  const { activeFolderId } = useActiveFolder()
  const [data, setData] = useState<ContextOverview | null>(null)
  const [loading, setLoading] = useState(true)
  const [refreshing, setRefreshing] = useState(false)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(
    async (background = false) => {
      if (activeFolderId == null) {
        setData(null)
        setLoading(false)
        return
      }
      if (background) {
        setRefreshing(true)
      } else {
        setLoading(true)
      }
      try {
        setData(await specosContextOverview(activeFolderId))
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

  const updateProvider = (
    id: string,
    patch: Partial<ContextProviderConfig>
  ) => {
    setData((current) =>
      current
        ? {
            ...current,
            config: {
              ...current.config,
              providers: current.config.providers.map((provider) =>
                provider.id === id ? { ...provider, ...patch } : provider
              ),
            },
          }
        : current
    )
  }
  const updateLoadout = (id: string, patch: Partial<ContextLoadout>) => {
    setData((current) =>
      current
        ? {
            ...current,
            config: {
              ...current.config,
              loadouts: current.config.loadouts.map((loadout) =>
                loadout.id === id ? { ...loadout, ...patch } : loadout
              ),
            },
          }
        : current
    )
  }
  const save = async () => {
    if (!data || activeFolderId == null) return
    setSaving(true)
    try {
      await specosContextConfigSave(activeFolderId, data.config)
      await load(true)
      toast.success(t("saved"))
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setSaving(false)
    }
  }

  if (loading)
    return (
      <div className="h-full space-y-4 overflow-auto p-6" aria-busy="true">
        <Skeleton className="h-24 rounded-2xl" />
        <Skeleton className="h-64 rounded-2xl" />
      </div>
    )
  if (activeFolderId == null)
    return <Empty title={t("noWorkspace")} body={t("noWorkspaceHint")} />
  if (error && !data)
    return (
      <Empty title={t("loadFailed")} body={error}>
        <Button onClick={() => void load()}>{t("retry")}</Button>
      </Empty>
    )
  if (!data) return null
  const degraded = data.providerHealth.filter(
    (provider) => provider.status === "degraded"
  )

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
      {degraded.length ? (
        <div className="flex items-center gap-2 border-b bg-amber-500/10 px-6 py-2 text-sm text-amber-700 dark:text-amber-300">
          <ShieldAlert className="size-4" />
          {t("degraded", { count: degraded.length })}
        </div>
      ) : null}
      <Tabs defaultValue="overview" className="min-h-0 flex-1 p-4 md:p-6">
        <TabsList>
          <TabsTrigger value="overview">{t("overview")}</TabsTrigger>
          <TabsTrigger value="codebase">{t("codebase.title")}</TabsTrigger>
          <TabsTrigger value="memory">{t("memory.title")}</TabsTrigger>
          <TabsTrigger value="providers">{t("providers")}</TabsTrigger>
          <TabsTrigger value="loadouts">{t("loadouts")}</TabsTrigger>
          <TabsTrigger value="activity">{t("activity")}</TabsTrigger>
        </TabsList>
        <TabsContent value="overview" className="min-h-0 overflow-auto pt-3">
          <div className="grid gap-4 md:grid-cols-3">
            <Metric label={t("packages")} value={data.packages.length} />
            <Metric
              label={t("providers")}
              value={data.config.providers.length}
            />
            <Metric label={t("loadouts")} value={data.config.loadouts.length} />
          </div>
          <div className="mt-4 space-y-3">
            {data.packages.length === 0 ? (
              <Empty
                title={t("noPackages")}
                body={t("noPackagesHint")}
                compact
              />
            ) : (
              data.packages.map((pkg) => (
                <Card key={pkg.id} size="sm">
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Database className="size-4" />
                      {pkg.id}
                      <Badge
                        variant={
                          pkg.status === "degraded"
                            ? "destructive"
                            : "secondary"
                        }
                      >
                        {pkg.status}
                      </Badge>
                    </CardTitle>
                    <CardDescription>
                      Task #{pkg.taskId} · run {pkg.runSeq} · {pkg.items.length}{" "}
                      {t("items")}
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="flex flex-wrap gap-2">
                    {pkg.items.map((item) => (
                      <Badge key={item.id} variant="outline">
                        {item.title}
                      </Badge>
                    ))}
                  </CardContent>
                </Card>
              ))
            )}
          </div>
        </TabsContent>
        <TabsContent value="codebase" className="min-h-0 overflow-auto pt-3">
          <CodebaseSection folderId={activeFolderId} />
        </TabsContent>
        <TabsContent value="memory" className="min-h-0 overflow-auto pt-3">
          <MemorySection
            folderId={activeFolderId}
            providers={data.config.providers.filter(
              (provider) => provider.kind === "memory"
            )}
          />
        </TabsContent>
        <TabsContent value="providers" className="min-h-0 overflow-auto pt-3">
          <div className="mb-3 flex justify-end">
            <Button size="sm" onClick={() => void save()} disabled={saving}>
              {saving ? <Loader2 className="animate-spin" /> : <Save />}
              {t("save")}
            </Button>
          </div>
          <div className="grid gap-3 lg:grid-cols-2">
            {data.config.providers.length === 0 ? (
              <Empty
                title={t("noProviders")}
                body={t("noProvidersHint")}
                compact
              />
            ) : (
              data.config.providers.map((provider) => {
                const health = data.providerHealth.find(
                  (item) => item.id === provider.id
                )
                return (
                  <Card key={provider.id} size="sm">
                    <CardHeader>
                      <CardTitle className="flex items-center gap-2">
                        {provider.id}
                        <Badge
                          variant={
                            health?.status === "healthy"
                              ? "secondary"
                              : health?.status === "disabled"
                                ? "outline"
                                : "destructive"
                          }
                        >
                          {health?.status ?? "unknown"}
                        </Badge>
                      </CardTitle>
                      <CardDescription>
                        {provider.kind}
                        {health?.message ? ` · ${health.message}` : ""}
                      </CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <label className="grid gap-1 text-xs text-muted-foreground">
                        {t("endpoint")}
                        <Input
                          value={provider.endpoint ?? ""}
                          onChange={(event) =>
                            updateProvider(provider.id, {
                              endpoint: event.target.value || null,
                            })
                          }
                        />
                      </label>
                      <label className="grid gap-1 text-xs text-muted-foreground">
                        {t("secretEnv")}
                        <Input
                          value={provider.secretEnv ?? ""}
                          onChange={(event) =>
                            updateProvider(provider.id, {
                              secretEnv: event.target.value || null,
                            })
                          }
                        />
                      </label>
                      <div className="flex flex-wrap gap-4">
                        <label className="flex items-center gap-2 text-sm">
                          <Checkbox
                            checked={provider.enabled}
                            onCheckedChange={(value) =>
                              updateProvider(provider.id, {
                                enabled: value === true,
                              })
                            }
                          />
                          {t("enabled")}
                        </label>
                        <label className="flex items-center gap-2 text-sm">
                          <Checkbox
                            checked={provider.required}
                            onCheckedChange={(value) =>
                              updateProvider(provider.id, {
                                required: value === true,
                              })
                            }
                          />
                          {t("required")}
                        </label>
                      </div>
                    </CardContent>
                  </Card>
                )
              })
            )}
          </div>
        </TabsContent>
        <TabsContent value="loadouts" className="min-h-0 overflow-auto pt-3">
          <div className="mb-3 flex justify-end">
            <Button size="sm" onClick={() => void save()} disabled={saving}>
              {saving ? <Loader2 className="animate-spin" /> : <Save />}
              {t("save")}
            </Button>
          </div>
          <div className="grid gap-3 lg:grid-cols-2">
            {data.config.loadouts.map((loadout) => (
              <Card key={loadout.id} size="sm">
                <CardHeader>
                  <CardTitle>{loadout.name}</CardTitle>
                  <CardDescription>{loadout.id}</CardDescription>
                </CardHeader>
                <CardContent className="grid gap-3">
                  <div className="grid grid-cols-3 gap-2">
                    <label className="grid gap-1 text-xs text-muted-foreground">
                      {t("maxItems")}
                      <Input
                        type="number"
                        min={1}
                        max={64}
                        value={loadout.maxItems}
                        onChange={(event) =>
                          updateLoadout(loadout.id, {
                            maxItems: Number(event.target.value),
                          })
                        }
                      />
                    </label>
                    <label className="grid gap-1 text-xs text-muted-foreground">
                      {t("maxBytes")}
                      <Input
                        type="number"
                        min={1}
                        max={524288}
                        value={loadout.maxBytes}
                        onChange={(event) =>
                          updateLoadout(loadout.id, {
                            maxBytes: Number(event.target.value),
                          })
                        }
                      />
                    </label>
                    <label className="grid gap-1 text-xs text-muted-foreground">
                      {t("maxTokens")}
                      <Input
                        type="number"
                        min={1}
                        max={32000}
                        value={loadout.maxTokens}
                        onChange={(event) =>
                          updateLoadout(loadout.id, {
                            maxTokens: Number(event.target.value),
                          })
                        }
                      />
                    </label>
                  </div>
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("providerIds")}
                    <Input
                      value={loadout.providerIds.join(", ")}
                      onChange={(event) =>
                        updateLoadout(loadout.id, {
                          providerIds: csv(event.target.value),
                        })
                      }
                    />
                  </label>
                  <label className="grid gap-1 text-xs text-muted-foreground">
                    {t("sourcesEditor")}
                    <Textarea
                      className="min-h-36 font-mono text-xs"
                      value={loadout.sources
                        .map(
                          (source) =>
                            `${source.path} | ${source.kind} | ${source.required ? "required" : "optional"}`
                        )
                        .join("\n")}
                      onChange={(event) =>
                        updateLoadout(loadout.id, {
                          sources: parseSources(event.target.value),
                        })
                      }
                    />
                  </label>
                  <p className="text-xs text-muted-foreground">
                    {t("sourcesHint")}
                  </p>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>
        <TabsContent value="activity" className="min-h-0 overflow-auto pt-3">
          {data.activity.length === 0 ? (
            <Empty title={t("noActivity")} body={t("noActivityHint")} compact />
          ) : (
            <ol className="space-y-2">
              {data.activity.map((item) => (
                <li
                  key={item.id}
                  className="flex items-start justify-between rounded-xl border p-3"
                >
                  <div>
                    <div className="font-medium">{item.kind}</div>
                    <div className="text-xs text-muted-foreground">
                      {item.message ?? item.packageId ?? item.providerId}
                    </div>
                  </div>
                  <Badge
                    variant={
                      item.status === "blocked" ? "destructive" : "outline"
                    }
                  >
                    {item.status}
                  </Badge>
                </li>
              ))}
            </ol>
          )}
        </TabsContent>
      </Tabs>
    </div>
  )
}

function CodebaseSection({ folderId }: { folderId: number }) {
  const t = useTranslations("ContextSystem.codebase")
  const [state, setState] = useState<CodeIntelState | null>(null)
  const [loading, setLoading] = useState(true)
  const [busy, setBusy] = useState<string | null>(null)
  const [overridePath, setOverridePath] = useState("")
  // The upstream Graph UI runs on the backend host's loopback — only a
  // pure-desktop window can reach it (server mode and remote-desktop
  // windows are refused / would point at the wrong machine).
  const graphAvailable = isDesktop() && !isRemoteDesktopMode()

  const load = useCallback(async () => {
    setLoading(true)
    try {
      setState(await codeIntelligenceGetState(folderId))
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setLoading(false)
    }
  }, [folderId])

  useEffect(() => {
    void load()
  }, [load])

  const run = async (action: string, op: () => Promise<unknown>) => {
    setBusy(action)
    try {
      const result = await op()
      if (result && typeof result === "object" && "project" in result) {
        setState(result as CodeIntelState)
      } else {
        await load()
      }
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setBusy(null)
    }
  }

  if (loading)
    return (
      <div className="space-y-4" aria-busy="true">
        <Skeleton className="h-32 rounded-2xl" />
        <Skeleton className="h-32 rounded-2xl" />
      </div>
    )
  if (!state || !state.runtimeAvailable)
    return (
      <Empty title={t("unavailable")} body={t("unavailableHint")} compact />
    )

  const { install, project, records } = state
  const busySpinner = (action: string) =>
    busy === action ? <Loader2 className="animate-spin" /> : null

  return (
    <div className="grid gap-4 lg:grid-cols-2">
      <Card size="sm">
        <CardHeader>
          <CardTitle>{t("adapterTitle")}</CardTitle>
          <CardDescription>{t("adapterHint")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex flex-wrap items-center gap-2 text-sm">
            <Badge variant={install?.installed ? "secondary" : "outline"}>
              {install?.installed
                ? `${install.reportedVersion ?? "?"} (${t(
                    `source.${install.source ?? "managed"}`
                  )})`
                : t("notInstalled")}
            </Badge>
            <span className="text-xs text-muted-foreground">
              {t("pinned", { version: install?.pinnedVersion ?? "?" })}
            </span>
          </div>
          {install?.binaryPath ? (
            <p className="break-all text-xs text-muted-foreground">
              {install.binaryPath}
            </p>
          ) : null}
          {!install?.installed ? (
            <Button
              size="sm"
              disabled={busy != null}
              onClick={() => void run("install", codeIntelligenceInstall)}
            >
              {busySpinner("install")}
              {t("install")}
            </Button>
          ) : null}
          <div className="space-y-1 border-t pt-3">
            <label className="grid gap-1 text-xs text-muted-foreground">
              {t("overrideLabel")}
              <Input
                value={overridePath}
                placeholder={t("overridePlaceholder")}
                onChange={(event) => setOverridePath(event.target.value)}
              />
            </label>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                disabled={busy != null}
                onClick={() =>
                  void run("override", () =>
                    codeIntelligenceSetBinaryOverride(overridePath || null)
                  )
                }
              >
                {busySpinner("override")}
                {t("overrideApply")}
              </Button>
              {install?.source !== "managed" && install?.source != null ? (
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={busy != null}
                  onClick={() =>
                    void run("override-clear", () =>
                      codeIntelligenceSetBinaryOverride(null)
                    )
                  }
                >
                  {busySpinner("override-clear")}
                  {t("overrideClear")}
                </Button>
              ) : null}
            </div>
            <p className="text-xs text-muted-foreground">{t("overrideHint")}</p>
          </div>
        </CardContent>
      </Card>

      <Card size="sm">
        <CardHeader>
          <CardTitle className="flex items-center justify-between gap-2">
            {t("indexTitle")}
            <Switch
              checked={project.bound && project.enabled}
              disabled={busy != null || !install?.installed}
              onCheckedChange={(checked) =>
                void run("toggle", () =>
                  codeIntelligenceSetEnabled(folderId, checked)
                )
              }
              aria-label={t("indexTitle")}
            />
          </CardTitle>
          <CardDescription>{t("indexHint")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <Badge
              variant={
                project.phase === "ready"
                  ? "secondary"
                  : project.phase === "not_indexed"
                    ? "outline"
                    : "destructive"
              }
            >
              {phaseLabel(t, project.phase)}
            </Badge>
            {project.revision ? (
              <span className="font-mono text-xs text-muted-foreground">
                {project.revision.slice(0, 12)}
              </span>
            ) : null}
            {project.fileCount != null ? (
              <span className="text-xs text-muted-foreground">
                {t("fileCount", { count: project.fileCount })}
              </span>
            ) : null}
          </div>
          <div className="flex flex-wrap gap-2">
            <Button
              variant="outline"
              size="sm"
              disabled={busy != null || !project.bound || !project.enabled}
              onClick={() =>
                void run("reindex", () => codeIntelligenceReindex(folderId))
              }
            >
              {busySpinner("reindex") ?? <RefreshCw />}
              {t("reindex")}
            </Button>
            {graphAvailable ? (
              <Button
                variant="outline"
                size="sm"
                disabled={busy != null || !project.bound || !project.enabled}
                onClick={() =>
                  void run("graph", async () => {
                    const url = await codeIntelligenceOpenGraph()
                    toast.success(t("graphOpened", { url }))
                  })
                }
              >
                {busySpinner("graph") ?? <Waypoints />}
                {t("openGraph")}
              </Button>
            ) : null}
          </div>
        </CardContent>
      </Card>

      <Card size="sm" className="lg:col-span-2">
        <CardHeader>
          <CardTitle>{t("recordsTitle")}</CardTitle>
          <CardDescription>{t("recordsHint")}</CardDescription>
        </CardHeader>
        <CardContent>
          {records.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t("noRecords")}</p>
          ) : (
            <ul className="space-y-2">
              {records.map((record) => (
                <li
                  key={record.repoPath}
                  className="flex flex-wrap items-center justify-between gap-2 rounded-xl border p-3 text-sm"
                >
                  <div className="min-w-0">
                    <div className="truncate font-mono text-xs">
                      {record.repoPath}
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {record.worktree
                        ? t("worktreeRecord", { taskId: record.taskId ?? "?" })
                        : t("baseRecord")}
                    </div>
                  </div>
                  <Badge variant={record.enabled ? "secondary" : "outline"}>
                    {record.enabled ? t("enabledBadge") : t("disabledBadge")}
                  </Badge>
                </li>
              ))}
            </ul>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

function MemorySection({
  folderId,
  providers,
}: {
  folderId: number
  providers: ContextProviderConfig[]
}) {
  const t = useTranslations("ContextSystem.memory")
  const [providerId, setProviderId] = useState<string | null>(null)
  const activeProvider =
    providers.find((provider) => provider.id === providerId) ?? providers[0]

  // ── Provider test ──
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<MemoryProviderTestResult | null>(
    null
  )
  const [testError, setTestError] = useState<string | null>(null)

  // ── Delivery list ──
  const [deliveries, setDeliveries] = useState<MemoryDeliveryInfo[] | null>(
    null
  )
  const [deliveriesLoading, setDeliveriesLoading] = useState(true)
  const [deliveriesError, setDeliveriesError] = useState<string | null>(null)
  const [retryingId, setRetryingId] = useState<number | null>(null)

  // ── Recall preview ──
  const [query, setQuery] = useState("")
  const [previewing, setPreviewing] = useState(false)
  const [preview, setPreview] = useState<MemoryRecallPreview | null>(null)
  const [previewError, setPreviewError] = useState<string | null>(null)

  const loadDeliveries = useCallback(async () => {
    setDeliveriesLoading(true)
    try {
      setDeliveries(await specosMemoryDeliveryList(folderId))
      setDeliveriesError(null)
    } catch (cause) {
      // Keep last-good rows; surface the transport error beside them.
      setDeliveriesError(toErrorMessage(cause))
    } finally {
      setDeliveriesLoading(false)
    }
  }, [folderId])

  useEffect(() => {
    void loadDeliveries()
  }, [loadDeliveries])

  useEffect(() => {
    if (activeProvider && providerId == null) setProviderId(activeProvider.id)
  }, [activeProvider, providerId])

  const runTest = async () => {
    if (!activeProvider) return
    setTesting(true)
    setTestError(null)
    try {
      setTestResult(await specosMemoryProviderTest(folderId, activeProvider.id))
    } catch (cause) {
      // Last-good result is retained; the failure is shown next to it.
      setTestError(toErrorMessage(cause))
    } finally {
      setTesting(false)
    }
  }

  const retry = async (id: number) => {
    setRetryingId(id)
    try {
      const updated = await specosMemoryDeliveryRetry(id)
      setDeliveries(
        (current) =>
          current?.map((row) => (row.id === updated.id ? updated : row)) ?? null
      )
    } catch (cause) {
      toast.error(toErrorMessage(cause))
    } finally {
      setRetryingId(null)
    }
  }

  const runPreview = async () => {
    if (!activeProvider || !query.trim()) return
    setPreviewing(true)
    setPreviewError(null)
    try {
      setPreview(
        await specosMemoryRecallPreview(folderId, activeProvider.id, query)
      )
    } catch (cause) {
      setPreview(null)
      setPreviewError(toErrorMessage(cause))
    } finally {
      setPreviewing(false)
    }
  }

  if (providers.length === 0)
    return (
      <Empty title={t("unconfigured")} body={t("unconfiguredHint")} compact />
    )

  const statusBadge = (result: MemoryProviderTestResult) => (
    <Badge
      variant={
        result.status === "healthy"
          ? "secondary"
          : result.status === "blocked"
            ? "outline"
            : "destructive"
      }
    >
      {t(`status.${result.status}`)}
    </Badge>
  )

  return (
    <div className="grid gap-4 lg:grid-cols-2">
      {/* Provider connection test */}
      <Card size="sm">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <BrainCircuit className="size-4" />
            {t("testTitle")}
          </CardTitle>
          <CardDescription>{t("testHint")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {providers.length > 1 ? (
            <label className="grid gap-1 text-xs text-muted-foreground">
              {t("provider")}
              <select
                className="h-9 rounded-md border bg-transparent px-2 text-sm"
                value={activeProvider?.id ?? ""}
                onChange={(event) => setProviderId(event.target.value)}
              >
                {providers.map((provider) => (
                  <option key={provider.id} value={provider.id}>
                    {provider.id}
                  </option>
                ))}
              </select>
            </label>
          ) : null}
          <Button
            size="sm"
            onClick={() => void runTest()}
            disabled={testing || !activeProvider}
          >
            {testing ? <Loader2 className="animate-spin" /> : <Play />}
            {t("runTest")}
          </Button>
          {testError ? (
            <p role="alert" className="text-sm text-destructive">
              {t("testFailed", { detail: testError })}
            </p>
          ) : null}
          {testResult ? (
            <div
              className="space-y-1 rounded-xl border p-3 text-sm"
              aria-label={t("testResult")}
            >
              <div className="flex flex-wrap items-center gap-2">
                {statusBadge(testResult)}
                {testResult.latencyMs != null ? (
                  <span className="text-xs text-muted-foreground">
                    {t("latency", { ms: testResult.latencyMs })}
                  </span>
                ) : null}
              </div>
              <div className="text-xs text-muted-foreground">
                {t("versionGate", {
                  version: testResult.version ?? "?",
                  state: testResult.versionMatch
                    ? t("gate.match")
                    : t("gate.mismatch"),
                })}
              </div>
              {testResult.errorKey ? (
                <div className="font-mono text-xs text-muted-foreground">
                  {testResult.errorKey}
                </div>
              ) : null}
            </div>
          ) : null}
        </CardContent>
      </Card>

      {/* Recall preview */}
      <Card size="sm">
        <CardHeader>
          <CardTitle>{t("previewTitle")}</CardTitle>
          <CardDescription>{t("previewHint")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <form
            className="flex flex-col gap-2 sm:flex-row"
            onSubmit={(event) => {
              event.preventDefault()
              void runPreview()
            }}
          >
            <Input
              value={query}
              placeholder={t("previewPlaceholder")}
              onChange={(event) => setQuery(event.target.value)}
              aria-label={t("previewPlaceholder")}
            />
            <Button
              type="submit"
              size="sm"
              variant="outline"
              disabled={previewing || !query.trim() || !activeProvider}
            >
              {previewing ? <Loader2 className="animate-spin" /> : null}
              {t("previewRun")}
            </Button>
          </form>
          {previewError ? (
            <p role="alert" className="text-sm text-destructive">
              {previewError}
            </p>
          ) : null}
          {preview ? (
            preview.hits.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                {t("previewEmpty")}
              </p>
            ) : (
              <ul className="space-y-2" aria-label={t("previewResults")}>
                {preview.hits.map((hit) => (
                  <li
                    key={`${hit.layer}-${hit.remoteId}-${hit.contentHash}`}
                    className="rounded-xl border p-3"
                  >
                    <div className="flex flex-wrap items-center gap-2 text-xs">
                      <Badge variant="outline">{hit.layer.toUpperCase()}</Badge>
                      <span className="font-mono text-muted-foreground">
                        {hit.remoteId}
                      </span>
                      {hit.score != null ? (
                        <span className="text-muted-foreground">
                          {t("score", { value: hit.score.toFixed(3) })}
                        </span>
                      ) : null}
                    </div>
                    {/* Remote content: plain untrusted text, already
                        truncated server-side to ~200 chars. */}
                    <p className="mt-1 break-words text-sm">{hit.preview}</p>
                  </li>
                ))}
              </ul>
            )
          ) : null}
        </CardContent>
      </Card>

      {/* Capture deliveries */}
      <Card size="sm" className="lg:col-span-2">
        <CardHeader>
          <CardTitle className="flex items-center justify-between gap-2">
            {t("deliveriesTitle")}
            <Button
              variant="ghost"
              size="sm"
              onClick={() => void loadDeliveries()}
              disabled={deliveriesLoading}
              aria-label={t("deliveriesRefresh")}
            >
              {deliveriesLoading ? (
                <Loader2 className="animate-spin" />
              ) : (
                <RefreshCw />
              )}
            </Button>
          </CardTitle>
          <CardDescription>{t("deliveriesHint")}</CardDescription>
        </CardHeader>
        <CardContent>
          {deliveriesLoading && deliveries == null ? (
            <div className="space-y-2" aria-busy="true">
              <Skeleton className="h-16 rounded-xl" />
              <Skeleton className="h-16 rounded-xl" />
            </div>
          ) : deliveries == null || deliveries.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t("deliveriesEmpty")}
            </p>
          ) : (
            <ul className="space-y-2" aria-label={t("deliveriesTitle")}>
              {deliveries.map((row) => (
                <li
                  key={row.id}
                  className="flex flex-wrap items-center justify-between gap-2 rounded-xl border p-3 text-sm"
                >
                  <div className="min-w-0">
                    <div className="font-medium">
                      {t("deliveryRow", {
                        provider: row.providerId,
                        task: row.taskId,
                        run: row.runSeq,
                      })}
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {t("attempts", { count: row.attempts })}
                      {row.acceptedCount != null
                        ? ` · ${t("accepted", { count: row.acceptedCount })}`
                        : ""}
                      {row.safeErrorCode ? ` · ${row.safeErrorCode}` : ""}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Badge
                      variant={
                        row.status === "delivered"
                          ? "secondary"
                          : row.status === "failed"
                            ? "destructive"
                            : "outline"
                      }
                    >
                      {t(`delivery.${row.status}`)}
                    </Badge>
                    {row.status === "failed" ? (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => void retry(row.id)}
                        disabled={retryingId === row.id}
                      >
                        {retryingId === row.id ? (
                          <Loader2 className="animate-spin" />
                        ) : null}
                        {t("retryDelivery")}
                      </Button>
                    ) : null}
                  </div>
                </li>
              ))}
            </ul>
          )}
          {deliveriesError ? (
            <p role="alert" className="mt-2 text-sm text-destructive">
              {t("deliveriesError", { detail: deliveriesError })}
            </p>
          ) : null}
        </CardContent>
      </Card>
    </div>
  )
}

type PhaseKey =
  | "phase.ready"
  | "phase.indexing"
  | "phase.not_indexed"
  | "phase.unknown"

const KNOWN_PHASES: Record<string, PhaseKey> = {
  ready: "phase.ready",
  indexing: "phase.indexing",
  not_indexed: "phase.not_indexed",
  unknown: "phase.unknown",
}

function phaseLabel(t: (key: PhaseKey) => string, phase: string): string {
  const key = KNOWN_PHASES[phase]
  return key ? t(key) : phase
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <Card size="sm">
      <CardHeader>
        <CardDescription>{label}</CardDescription>
        <CardTitle className="text-3xl">{value}</CardTitle>
      </CardHeader>
    </Card>
  )
}
function Empty({
  title,
  body,
  children,
  compact = false,
}: {
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
          <Database />
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

function parseSources(value: string): ContextLoadout["sources"] {
  return value
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const [path, kind = "project", requirement = "optional"] = line
        .split("|")
        .map((part) => part.trim())
      return { path, kind, required: requirement === "required" }
    })
}
