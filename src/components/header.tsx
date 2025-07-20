"use client";

import { GrpcRepoContext } from "@/interfaces/react/contexts/grpcRepoContext";
import { useGrpcConnect } from "@/interfaces/react/hooks/useGrpcConnect";
import { Home, Settings } from "lucide-react";
import { useContext, useEffect } from "react";
import { Card, CardContent } from "./ui/card";

export function Header() {
  const grpcRepo = useContext(GrpcRepoContext);
  const { status, connect } = useGrpcConnect(grpcRepo);

  useEffect(() => {
    connect();
  }, [connect]);

  return (
    <header className="flex flex-row items-center justify-between p-4">
      <div className="grow">
        {/* <h1 className="text-2xl font-bold">roomtemp</h1> */}
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
