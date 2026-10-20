import { SendIcon, SkipForwardIcon } from 'lucide-react'
import { useState, type ReactNode } from 'react'
import { toast } from 'sonner'
import { useSettings } from '@/apis/settings'
import { useSendApplication, useSkipStartup, type TStartupInsight } from '@/apis/startups'
import { ConfirmDialog } from '@/components/features/shared/confirm-dialog'
import { Button } from '@/components/ui/button'
import { FieldDescription, FieldGroup } from '@/components/ui/field'
import { ScrollArea } from '@/components/ui/scroll-area'
import { SheetFooter } from '@/components/ui/sheet'
import { Spinner } from '@/components/ui/spinner'
import { errorMessage } from '@/libs/axios'
import { EmailFields } from './email-fields'
import type { TEmailDraft } from './email-schema'
import { useEmailForm } from './use-email-form'

type TReviewEmailFormProps = {
  insight: TStartupInsight
  onDone: () => void
  children: ReactNode
}

export function ReviewEmailForm({ insight, onDone, children }: TReviewEmailFormProps) {
  const { data: settings } = useSettings()
  const send = useSendApplication()
  const skip = useSkipStartup()
  const [pending, setPending] = useState<TEmailDraft | null>(null)
  const { startup } = insight

  const form = useEmailForm(insight.email, setPending)

  const blocker = !settings?.mailer_configured
    ? `${settings?.mailer_provider ?? 'Mailer'} is not configured`
    : !settings.resume_loaded
      ? 'Resume file is missing'
      : insight.status === 'APPLIED'
        ? 'Already applied'
        : null

  const confirmSend = () => {
    if (!pending) {
      return
    }
    send.mutate(
      { startup, email: pending },
      {
        onSuccess: ({ data }) => {
          if (data.outcome.kind === 'applied') {
            toast.success(`Application sent to ${pending.to}`)
            onDone()
          } else {
            toast.error('Send failed', { description: data.outcome.detail })
          }
        },
        onError: (error) => toast.error('Send failed', { description: errorMessage(error) }),
        onSettled: () => setPending(null),
      },
    )
  }

  const skipStartup = () =>
    skip.mutate(startup, {
      onSuccess: () => {
        toast(`${startup.name} marked as skipped`)
        onDone()
      },
      onError: (error) => toast.error('Could not skip', { description: errorMessage(error) }),
    })

  return (
    <form
      className="flex min-h-0 flex-1 flex-col"
      onSubmit={(event) => {
        event.preventDefault()
        void form.handleSubmit()
      }}
    >
      <ScrollArea className="min-h-0 flex-1">
        <FieldGroup className="gap-5 p-4">
          {children}
          <EmailFields form={form} />
        </FieldGroup>
      </ScrollArea>
      <SheetFooter className="flex-row items-center border-t">
        <FieldDescription className="mr-auto text-xs">{blocker}</FieldDescription>
        <Button
          type="button"
          variant="outline"
          disabled={skip.isPending || insight.status === 'APPLIED'}
          onClick={skipStartup}
        >
          <SkipForwardIcon />
          Skip
        </Button>
        <Button type="submit" disabled={Boolean(blocker) || send.isPending}>
          {send.isPending ? <Spinner /> : <SendIcon />}
          Send application
        </Button>
      </SheetFooter>
      <ConfirmDialog
        open={pending !== null}
        onOpenChange={(open) => !open && setPending(null)}
        title={`Send to ${startup.name}?`}
        description={`The email goes to ${pending?.to ?? ''} with your resume attached. This cannot be undone.`}
        confirmLabel="Send now"
        onConfirm={confirmSend}
      />
    </form>
  )
}
