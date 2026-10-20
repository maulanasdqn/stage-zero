import { CheckCircle2Icon, CircleAlertIcon } from 'lucide-react'
import type { THuntRun } from '@/apis/hunt'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Badge } from '@/components/ui/badge'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import { Progress } from '@/components/ui/progress'
import { Spinner } from '@/components/ui/spinner'

function StateBadge({ run }: { run: THuntRun }) {
  if (run.running) {
    return (
      <Badge variant="outline" className="border-primary/30 bg-primary/10 text-primary">
        <Spinner />
        Running
      </Badge>
    )
  }
  if (run.error) {
    return (
      <Badge variant="outline" className="border-destructive/30 bg-destructive/10 text-destructive">
        <CircleAlertIcon />
        Failed
      </Badge>
    )
  }
  return (
    <Badge variant="outline" className="border-success/30 bg-success/10 text-success">
      <CheckCircle2Icon />
      Finished
    </Badge>
  )
}

export function HuntStatus({ run, className }: { run: THuntRun; className?: string }) {
  const criteria = run.criteria
  if (!criteria) {
    return null
  }
  const processed = run.events.filter((event) => event.outcome.kind !== 'already_processed').length
  const target = Math.min(criteria.limit ?? run.discovered ?? 0, run.discovered ?? Infinity)
  const percent = target > 0 ? Math.min(100, Math.round((processed / target) * 100)) : 0
  const summary = [
    criteria.batch,
    `team ${criteria.team_size.min}-${criteria.team_size.max}`,
    criteria.industry,
    criteria.dry_run ? 'dry run' : 'live send',
  ]
    .filter(Boolean)
    .join(' · ')

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle>Current run</CardTitle>
        <CardDescription>{summary}</CardDescription>
        <CardAction>
          <StateBadge run={run} />
        </CardAction>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <Field>
          <FieldLabel htmlFor="hunt-progress" className="tabular-nums">
            {processed} of {target} processed
          </FieldLabel>
          <Progress id="hunt-progress" value={run.running || run.report ? percent : 0} />
          {run.discovered !== null && (
            <FieldDescription className="tabular-nums">
              {run.discovered} startups matched in the batch
            </FieldDescription>
          )}
        </Field>
        {run.error && (
          <Alert variant="destructive">
            <AlertTitle>Hunt failed</AlertTitle>
            <AlertDescription>{run.error}</AlertDescription>
          </Alert>
        )}
        {run.report?.daily_limit_reached && (
          <Alert>
            <CircleAlertIcon />
            <AlertTitle>Daily send limit reached</AlertTitle>
            <AlertDescription>Run the hunt again tomorrow to continue.</AlertDescription>
          </Alert>
        )}
      </CardContent>
    </Card>
  )
}
