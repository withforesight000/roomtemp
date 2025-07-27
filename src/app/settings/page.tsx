"use client";

import React, { useContext, useEffect, useMemo } from "react";
import { SettingsRepositoryImpl } from "@/interfaces/repositories/settings";
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
import { GrpcRepoContext } from "@/interfaces/react/contexts/grpcRepoContext";

export default function SettingsPage() {
  const settingsRepo = useMemo(() => new SettingsRepositoryImpl(), []);
  const { connect } = useContext(GrpcRepoContext);
  const { state: settingsState, dispatch, load, save } = useSettings(settingsRepo);

  const saveSettings = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    // The save logic is handled in the useSettings hook
    await save();
    await connect(); // Reconnect to apply new settings
  };

  useEffect(() => {
    load();
  }, []);

  return (
    <Card className="w-full">
      <form onSubmit={saveSettings}>
        <CardHeader className="my-4">
          <CardTitle>Settings</CardTitle>
        </CardHeader>
        <CardContent className="my-4">
          <div className="grid w-full items-center gap-4">
            <div className="flex flex-col space-y-1.5">
              <Label htmlFor="name">URL</Label>
              <p className="text-sm text-muted-foreground">
                Enter the URL of the gRPC endpoint which provides the graph
                data:
              </p>
              <Input
                id="name"
                placeholder="https://example.com/grpc"
                value={settingsState.url}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                  dispatch({
                    type: "SET_FIELD",
                    payload: { ...settingsState, url: e.currentTarget.value },
                  });
                }}
              />
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
                value={settingsState.accessToken}
                onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                  dispatch({
                    type: "SET_FIELD",
                    payload: { ...settingsState, accessToken: e.currentTarget.value },
                  });
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
        </CardContent>

        <CardFooter className="flex justify-between mt-4">
          <Button type="submit" className="cursor-pointer">
            Update
          </Button>
        </CardFooter>
      </form>
    </Card>
  );
}
