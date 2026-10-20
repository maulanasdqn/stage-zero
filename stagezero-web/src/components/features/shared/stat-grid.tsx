import type { LucideIcon } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from '@/components/ui/item'
import { Skeleton } from '@/components/ui/skeleton'
import { cn } from '@/lib/utils'

export type TStat = {
  label: string
  value: number | undefined
  icon: LucideIcon
}

type TStatGridProps = {
  stats: TStat[]
  className?: string
}

export function StatGrid({ stats, className }: TStatGridProps) {
  return (
    <Card size="sm" className={className}>
      <CardContent
        className={cn(
          'grid grid-cols-2 gap-2',
          stats.length > 4 ? 'sm:grid-cols-3 lg:grid-cols-6' : 'lg:grid-cols-4',
        )}
      >
        {stats.map(({ label, value, icon: Icon }) => (
          <Item key={label} variant="muted" size="sm">
            <ItemContent>
              <ItemDescription>{label}</ItemDescription>
              <ItemTitle className="font-mono text-2xl font-semibold tabular-nums">
                {value === undefined ? <Skeleton className="h-7 w-10" /> : value}
              </ItemTitle>
            </ItemContent>
            <ItemActions className="self-start">
              <Icon className="text-muted-foreground size-4" />
            </ItemActions>
          </Item>
        ))}
      </CardContent>
    </Card>
  )
}
