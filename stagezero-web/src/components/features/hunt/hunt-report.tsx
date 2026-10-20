import {
  CircleSlashIcon,
  EyeIcon,
  GlobeLockIcon,
  RepeatIcon,
  SendIcon,
  UserXIcon,
} from 'lucide-react'
import type { THuntReport } from '@/apis/hunt'
import { StatGrid } from '@/components/features/shared/stat-grid'

export function HuntReport({ report, className }: { report: THuntReport; className?: string }) {
  return (
    <StatGrid
      className={className}
      stats={[
        { label: 'Sent', value: report.applied, icon: SendIcon },
        { label: 'Previewed', value: report.previewed, icon: EyeIcon },
        { label: 'No contact', value: report.no_contact, icon: UserXIcon },
        { label: 'Already done', value: report.already_processed, icon: RepeatIcon },
        { label: 'No website', value: report.skipped_no_website, icon: GlobeLockIcon },
        { label: 'Send failures', value: report.send_failed, icon: CircleSlashIcon },
      ]}
    />
  )
}
