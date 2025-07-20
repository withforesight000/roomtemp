"use client";

import { GrpcRepoContext } from "@/interfaces/react/contexts/grpcRepoContext";
import { useGrpcConnect } from "@/interfaces/react/hooks/useGrpcConnect";
import { Home, MoveLeft, Settings } from "lucide-react";
import { useContext, useEffect } from "react";
import { Card, CardContent } from "./ui/card";
import { usePathname, useRouter } from "next/navigation";

export function Header() {
  const grpcRepo = useContext(GrpcRepoContext);
  const { status, connect } = useGrpcConnect(grpcRepo);

  const pathname = usePathname();
  const router = useRouter();

  useEffect(() => {
    connect();
  }, [connect]);

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
      <div className="grow">
      </div>
      <Card className="w-64 h-6 mx-6">
        <CardContent className="flex items-center justify-center h-full">
          <p>{status}</p>
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
