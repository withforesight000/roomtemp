"use client";

import { AlertCircle } from "lucide-react"
import { useReducer, useEffect } from "react"
import { invoke } from '@tauri-apps/api/core';
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

type State = {
  url: string | undefined
  accessToken: string | undefined
  error?: string
}

function reducer(state: State, action: { type: string; payload: { url?: string; accessToken?: string; error?: string } }): State {
  switch (action.type) {
    case "LOAD_FIELDS":
      return { ...state, url: action.payload.url, accessToken: action.payload.accessToken }
    case "SET_URL_FIELD":
      return { ...state, url: action.payload.url }
    case "SET_ACCESS_TOKEN_FIELD":
      return { ...state, accessToken: action.payload.accessToken }
    case "SET_ERROR":
      return { ...state, error: action.payload.error }
    default:
      return state
  }
}

export default function CardWithForm() {
  const [state, dispatch] = useReducer(reducer, {
    url: "",
    accessToken: "",
    error: undefined,
  } as State);

  useEffect(() => {
    // Fetch initial settings from the server or local storage
    const fetchSettings = async () => {
      try {
        const response = await invoke<State>('get_settings')
        console.log("get_settings_", response)
        dispatch({ type: "LOAD_FIELDS", payload: { url: response.url, accessToken: response.accessToken } })
      } catch (error: unknown) {
        console.log('Error fetching settings:', error);
        if (error instanceof Error) {
          dispatch({ type: "SET_ERROR", payload: { error: error.message } })
        } else if (typeof error === "string") {
          dispatch({ type: "SET_ERROR", payload: { error: error } })
        } else {
          dispatch({ type: "SET_ERROR", payload: { error: "Unknown error" } })
        }
      }
    }

    fetchSettings()
  }, []);

  const saveSettings = async (e: React.FormEvent<HTMLFormElement>) => {
    console.log("saveSettings")
    e.preventDefault()
    try {
      const response = await invoke('set_settings', { url: state.url, accessToken: state.accessToken })
      console.log(response)
    } catch (error: unknown) {
      if (error instanceof Error) {
        dispatch({ type: "SET_ERROR", payload: { error: error.message } })
      } else if (typeof error === "string") {
        dispatch({ type: "SET_ERROR", payload: { error: error } })
      } else {
        dispatch({ type: "SET_ERROR", payload: { error: "Unknown error" } })
      }
    }
  }

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
                Enter the URL of the gRPC endpoint which provides the graph data:
              </p>
              <Input id="name" placeholder="https://example.com/grpc" value={state.url}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                        dispatch({ type: "SET_URL_FIELD", payload: { url: e.currentTarget.value } })
                      }}
              />
            </div>

            <div className="flex flex-col space-y-1.5">
              <Label htmlFor="name">Access Token</Label>
              <p className="text-sm text-muted-foreground">
                Enter the access token for the gRPC endpoint:
              </p>
              <Input id="access-token" placeholder="Your Access Token" type="password" value={state.accessToken}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                        dispatch({ type: "SET_ACCESS_TOKEN_FIELD", payload: { "accessToken": e.currentTarget.value } })
                      }}
              />
            </div>

            {state.error && (
              <Alert variant="destructive" className="flex flex-col space-y-1.5">
                <AlertCircle className="h-4 w-4" />
                <AlertTitle>Error!</AlertTitle>
                <AlertDescription>
                  {state.error}
                </AlertDescription>
              </Alert>
            )}
          </div>
        </CardContent>

        <CardFooter className="flex justify-between mt-4">
          <Button variant="outline">Cancel</Button>
          <Button type="submit">Update</Button>
        </CardFooter>
      </form>
    </Card>
  )
}
