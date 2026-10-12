"use client";

import type React from "react";
import { useSettings } from "@/interfaces/react/hooks/useSettings";
import { AlertCircle } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useFrontend } from "@/interfaces/react/contexts/grpcRepoContext";
import { Checkbox } from "@/components/ui/checkbox";

export default function SettingsPage() {
  const { operations, saveSettings, state: connectionState } = useFrontend();
  const { state: settingsState, setField, save } = useSettings(operations, saveSettings);
  const submit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    void save();
  };

  return (
    <Card className="w-full">
      <form onSubmit={submit}>
        <CardHeader className="my-4">
          <CardTitle>Settings</CardTitle>
        </CardHeader>
        <CardContent className="my-4">
          <fieldset disabled={connectionState.isSaving}>
            <div className="w-full items-center gap-4">
              <div className="flex flex-col space-y-1.5">
                <Label htmlFor="name">URL</Label>
                <p className="text-sm text-muted-foreground">
                  Enter the URL of the gRPC endpoint which provides the graph
                  data:
                </p>
                <Input
                  id="name"
                  placeholder="https://example.com/grpc"
                  value={settingsState.values.url}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                    setField({ field: "url", value: e.currentTarget.value });
                  }}
                />
                <div className="flex flex-col ml-4 mt-1.5 mb-8 space-y-1.5">
                  <div className="flex flex-row items-center space-x-2 space-y-1.5">
                    <Checkbox
                      id="use-proxies"
                      checked={settingsState.values.useProxies}
                      onCheckedChange={(checked: boolean) => {
                        setField({ field: "useProxies", value: checked });
                      }}
                    />
                    <Label htmlFor="use-proxies">Use a Proxy Server</Label>
                  </div>
                  <div
                    className={`ml-6 space-y-1.5 transition-opacity duration-150 ${
                      settingsState.values.useProxies
                        ? "opacity-100"
                        : "opacity-50 pointer-events-none"
                    }`}
                    aria-disabled={!settingsState.values.useProxies}
                  >
                    <Label
                      htmlFor="proxy_url"
                      className={
                        settingsState.values.useProxies ? "" : "text-muted-foreground"
                      }
                    >
                      Proxy URL
                    </Label>
                    <p className="text-sm text-muted-foreground">
                      Enter the proxy server URL:
                    </p>
                    <Input
                      id="proxy_url"
                      placeholder="http://proxy.example.com:8080"
                      value={settingsState.values.proxyUrl}
                      disabled={!settingsState.values.useProxies}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                        setField({ field: "proxyUrl", value: e.currentTarget.value });
                      }}
                    />
                  </div>
                </div>
              </div>

              <div className="flex flex-col space-y-1.5">
                <Label htmlFor="name">Access Token</Label>
                <p className="text-sm text-muted-foreground">
                  Enter the access token for the gRPC endpoint:
                </p>
                <Input
                  id="access-token"
                  placeholder="Your Access Token"
                  type="password"
                  value={settingsState.values.accessToken}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                    setField({ field: "accessToken", value: e.currentTarget.value });
                  }}
                />
              </div>

              {settingsState.error && (
                <Alert
                  variant="destructive"
                  className="flex flex-col space-y-1.5"
                >
                  <AlertCircle className="h-4 w-4" />
                  <AlertTitle>Error!</AlertTitle>
                  <AlertDescription>{settingsState.error}</AlertDescription>
                </Alert>
              )}
            </div>
          </fieldset>
        </CardContent>

        <div className="flex">
          <CardFooter className="flex justify-between mt-4">
            <Button disabled={connectionState.isSaving} type="submit" className="cursor-pointer">
              Update
            </Button>
          </CardFooter>
        </div>
      </form>
    </Card>
  );
}
