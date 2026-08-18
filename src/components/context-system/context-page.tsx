"use client"

import { useCallback, useEffect, useState } from "react"
import { useTranslations } from "next-intl"
import { toast } from "sonner"
import {
  Database,
  Loader2,
  Plus,
  RefreshCw,
  Save,
  ShieldAlert,
} from "lucide-react"
import { useActiveFolder } from "@/contexts/active-folder-context"
import { specosContextConfigSave, specosContextOverview } from "@/lib/api"
import { toErrorMessage } from "@/lib/app-error"
import type {
  ContextLoadout,
  ContextOverview,
  ContextProviderConfig,
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
  const addProvider = () => {
    setData((current) => {
      if (!current) return current
      const n = current.config.providers.length + 1
      return {
        ...current,
        config: {
          ...current.config,
          providers: [
            ...current.config.providers,
            {
              id: `tencent-memory-${n}`,
              kind: "tencent-memory",
              endpoint: "http://127.0.0.1:8125",
              secretEnv: null,
              enabled: true,
              required: false,
              capabilities: ["memory", "wiki", "skill", "codegraph"],
            },
          ],
        },
      }
    })
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
        <TabsContent value="providers" className="min-h-0 overflow-auto pt-3">
          <div className="mb-3 flex justify-end gap-2">
            <Button variant="outline" size="sm" onClick={addProvider}>
              <Plus />
              {t("addProvider")}
            </Button>
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
