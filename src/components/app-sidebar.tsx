'use client';

import { Menu, Calendar, Home, Inbox, Search, Settings } from "lucide-react"

import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar
} from "@/components/ui/sidebar"

// Menu items.
const items = [
  {
    title: "Home",
    url: "#",
    icon: Home,
  },
  {
    title: "Inbox",
    url: "#",
    icon: Inbox,
  },
  {
    title: "Calendar",
    url: "#",
    icon: Calendar,
  },
  {
    title: "Search",
    url: "#",
    icon: Search,
  },
  {
    title: "Settings",
    url: "#",
    icon: Settings,
  },
]

function CustomTrigger() {
    const { toggleSidebar } = useSidebar()

    return <button className="p-3 cursor-pointer" onClick={toggleSidebar}><Menu /></button>
}

export function AppSidebar() {
  return (
    <Sidebar collapsible="icon">
        <CustomTrigger/>
        <SidebarContent>
            <SidebarGroup>
            <SidebarGroupLabel>Application</SidebarGroupLabel>
            <SidebarGroupContent>
                <SidebarMenu>
                {items.map((item) => (
                    <SidebarMenuItem key={item.title}>
                    <SidebarMenuButton asChild>
                        <a href={item.url}>
                        <item.icon />
                        <span>{item.title}</span>
                        </a>
                    </SidebarMenuButton>
                    </SidebarMenuItem>
                ))}
                </SidebarMenu>
            </SidebarGroupContent>
            </SidebarGroup>
        </SidebarContent>
    </Sidebar>
//     <Collapsible defaultOpen className="group/collapsible">
//     <SidebarGroup>
//       <SidebarGroupLabel asChild>
//         <CollapsibleTrigger>
//           Help
//           <ChevronDown className="ml-auto transition-transform group-data-[state=open]/collapsible:rotate-180" />
//         </CollapsibleTrigger>
//       </SidebarGroupLabel>
//       <CollapsibleContent>
//         <SidebarGroupContent />
//       </CollapsibleContent>
//     </SidebarGroup>
//   </Collapsible>
  )
}
