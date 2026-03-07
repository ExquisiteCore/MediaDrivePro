import { ChevronRight, Home } from 'lucide-react'

export interface BreadcrumbItem {
  id: string | null
  name: string
}

interface BreadcrumbProps {
  items: BreadcrumbItem[]
  onNavigate: (folderId: string | null) => void
}

export default function Breadcrumb({ items, onNavigate }: BreadcrumbProps) {
  return (
    <nav className="flex items-center gap-1 text-sm">
      <button
        onClick={() => onNavigate(null)}
        className="flex items-center gap-1 text-gray-500 hover:text-blue-600 transition-colors"
      >
        <Home className="w-4 h-4" />
      </button>
      {items.map((item, idx) => (
        <span key={item.id ?? 'root'} className="flex items-center gap-1">
          <ChevronRight className="w-4 h-4 text-gray-400" />
          {idx === items.length - 1 ? (
            <span className="font-medium text-gray-900">{item.name}</span>
          ) : (
            <button
              onClick={() => onNavigate(item.id)}
              className="text-gray-500 hover:text-blue-600 transition-colors"
            >
              {item.name}
            </button>
          )}
        </span>
      ))}
    </nav>
  )
}
