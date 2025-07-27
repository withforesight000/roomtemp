"use client";

import { GrpcRepoContext } from "@/interfaces/react/contexts/grpcRepoContext";
import { CircleCheck, CircleX, Home, MoveLeft, Settings } from "lucide-react";
import { useContext } from "react";
import { Card, CardContent } from "./ui/card";
import { usePathname, useRouter } from "next/navigation";

export function Header() {
  const { state: grpcConnectState } = useContext(GrpcRepoContext);
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
      <Card className="w-80 h-6 mx-6">
        <CardContent className="flex items-center justify-center h-full">
          {!grpcConnectState.isLoading && (
            <p className="flex items-center gap-2">
              {grpcConnectState.status}
              {grpcConnectState.hasError ? (
                <CircleX className="h-4 w-4 text-red-500" />
              ) : (
                <CircleCheck className="h-4 w-4 text-green-500" />
              )}
            </p>
          )}
        </CardContent>
      </Card>
      <nav className="flex-none">
        <ul className="flex space-x-4">
          <li>
            <a href="/" className="hover:underline">
              <Home className="h-6 w-6" />
            </a>
          </li>
          <li>
            <a href="/settings" className="hover:underline">
              <Settings className="h-6 w-6" />
            </a>
          </li>
        </ul>
      </nav>
    </header>
  );
}
