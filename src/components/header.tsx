"use client";

import { useFrontend } from "@/interfaces/react/contexts/grpcRepoContext";
import { CircleCheck, CircleX, Home, MoveLeft, Settings } from "lucide-react";
import { Card, CardContent } from "./ui/card";
import { usePathname, useRouter } from "next/navigation";

export function Header() {
  const { state: grpcConnectState, connect } = useFrontend();
  const pathname = usePathname();
  const router = useRouter();
  const showBackButton = pathname !== "/";

  return (
    <header className="flex flex-row items-center justify-between p-4">
      <div className="flex items-center">
        {showBackButton && (
          <button
            onClick={() => router.back()}
            className="mr-4 p-1 rounded cursor-pointer"
          >
            <MoveLeft className="h-6 w-6" />
          </button>
        )}
      </div>
      <div className="grow"></div>
      <Card className="max-w-xl min-w-0 min-h-10 mx-6 py-0">
        <CardContent className="flex items-center justify-center min-h-10 py-2">
          <div role="status" aria-live="polite" className="flex flex-col items-center gap-2">
            <p className="flex items-center gap-2">
              {grpcConnectState.isSaving ? "Saving settings…" : grpcConnectState.status}
              {!grpcConnectState.isSaving && grpcConnectState.phase === "ready" && <CircleCheck className="h-4 w-4 shrink-0 text-green-500" />}
              {!grpcConnectState.isSaving && grpcConnectState.phase === "error" && <CircleX className="h-4 w-4 shrink-0 text-red-500" />}
            </p>
            {grpcConnectState.phase === "error" && (
              <button disabled={grpcConnectState.isSaving} onClick={() => void connect()} className="underline disabled:opacity-50">
                Retry Connection
              </button>
            )}
          </div>
        </CardContent>
      </Card>
      <nav className="flex-none">
        <ul className="flex space-x-4">
          <li>
            <a href="/" aria-label="Home" className="inline-flex hover:underline">
              <Home className="h-6 w-6" />
            </a>
          </li>
          <li>
            <a href="/settings" aria-label="Settings" className="inline-flex hover:underline">
              <Settings className="h-6 w-6" />
            </a>
          </li>
        </ul>
      </nav>
    </header>
  );
}
