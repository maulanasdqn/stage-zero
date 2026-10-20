import { cn } from '@/lib/utils'
import { Item, ItemContent, ItemDescription, ItemTitle } from '@/components/ui/item'

type TPageHeaderProps = {
  title: string
  description: string
  className?: string
}

export function PageHeader({ title, description, className }: TPageHeaderProps) {
  return (
    <Item className={cn('w-auto p-0', className)}>
      <ItemContent>
        <ItemTitle className="text-xl font-semibold tracking-tight">{title}</ItemTitle>
        <ItemDescription className="line-clamp-none">{description}</ItemDescription>
      </ItemContent>
    </Item>
  )
}
