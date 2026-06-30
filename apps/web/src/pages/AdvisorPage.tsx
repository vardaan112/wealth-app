import { Loader2, Sparkles } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { useMutation, useQuery } from 'urql'
import {
  CHAT_MESSAGES_QUERY,
  SEND_CHAT_MESSAGE_MUTATION,
} from '../graphql/queries'
import type {
  ChatMessagesQuery,
  SendChatMessageMutation,
} from '../graphql/types'

const DAILY_BRIEFING_PROMPT =
  'Review my portfolio holdings and give buy/hold/sell alerts for today. Be specific per symbol.'

function formatBriefingDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  })
}

export function AdvisorPage() {
  const [input, setInput] = useState('')
  const [localError, setLocalError] = useState<string | null>(null)
  const [messages, setMessages] = useState<
    ChatMessagesQuery['chatMessages']['messages']
  >([])
  const [lastBriefingAt, setLastBriefingAt] = useState<string | null>(null)
  const bottomRef = useRef<HTMLDivElement>(null)

  const [historyResult, reexecuteHistory] = useQuery<ChatMessagesQuery>({
    query: CHAT_MESSAGES_QUERY,
  })

  const [sendResult, sendChatMessage] = useMutation<SendChatMessageMutation>(
    SEND_CHAT_MESSAGE_MUTATION,
  )

  useEffect(() => {
    if (historyResult.data?.chatMessages) {
      setMessages(historyResult.data.chatMessages.messages)
      setLastBriefingAt(historyResult.data.chatMessages.lastBriefingAt ?? null)
    }
  }, [historyResult.data])

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages, sendResult.fetching])

  const isLoading = sendResult.fetching || historyResult.fetching

  async function handleSend(content: string, isBriefing = false) {
    const trimmed = content.trim()
    if (!trimmed || sendResult.fetching) {
      return
    }

    setLocalError(null)
    setInput('')

    const result = await sendChatMessage({ input: { content: trimmed, isBriefing } })

    if (result.error) {
      setLocalError(result.error.message)
      setInput(trimmed)
      return
    }

    if (result.data?.sendChatMessage) {
      const { userMessage, assistantMessage } = result.data.sendChatMessage
      setMessages((current) => [...current, userMessage, assistantMessage])
      if (isBriefing) {
        setLastBriefingAt(assistantMessage.createdAt)
      }
    }

    reexecuteHistory({ requestPolicy: 'network-only' })
  }

  function handleSubmit(event: React.FormEvent) {
    event.preventDefault()
    void handleSend(input)
  }

  return (
    <div className="animate-rise mx-auto flex h-[calc(100dvh-7rem)] max-w-3xl flex-col gap-4 pt-6 md:h-[calc(100dvh-4rem)]">
      <header className="shrink-0">
        <h1 className="text-3xl font-semibold tracking-[-0.04em]">Advisor</h1>
        <p className="mt-2 text-sm text-muted">
          Ask follow-up questions about your portfolio. Answers use your synced
          holdings.
        </p>
        <div className="mt-4 flex flex-wrap items-center gap-3">
          <button
            type="button"
            disabled={isLoading}
            onClick={() => void handleSend(DAILY_BRIEFING_PROMPT, true)}
            className="inline-flex items-center gap-2 rounded-full border border-white/[0.08] bg-white/[0.03] px-4 py-2 text-sm text-text/90 transition-colors hover:bg-white/[0.06] disabled:opacity-50"
          >
            {sendResult.fetching ? (
              <Loader2 className="size-4 animate-spin" />
            ) : (
              <Sparkles className="size-4 text-accent" strokeWidth={1.75} />
            )}
            Generate daily briefing
          </button>
          {lastBriefingAt ? (
            <span className="text-xs text-muted">
              Last briefing: {formatBriefingDate(lastBriefingAt)}
            </span>
          ) : null}
        </div>
      </header>

      <div className="flex min-h-0 flex-1 flex-col rounded-2xl border border-white/[0.06] bg-white/[0.02]">
        <div className="flex-1 space-y-4 overflow-y-auto p-4 sm:p-5">
          {historyResult.error ? (
            <p className="text-sm text-red-300">{historyResult.error.message}</p>
          ) : null}
          {localError ? (
            <p className="rounded-xl border border-red-400/20 bg-red-400/10 px-4 py-3 text-sm text-red-200">
              {localError.includes('OPENAI_API_KEY')
                ? 'Add OPENAI_API_KEY to services/api/.env and restart the API.'
                : localError}
            </p>
          ) : null}
          {!historyResult.fetching && messages.length === 0 ? (
            <div className="flex h-full min-h-48 flex-col items-center justify-center text-center">
              <p className="text-sm text-muted">
                Start a conversation or generate today&apos;s briefing.
              </p>
            </div>
          ) : null}
          {messages.map((message) => {
            const isUser = message.role === 'user'
            return (
              <div
                key={message.id}
                className={isUser ? 'flex justify-end' : 'flex justify-start'}
              >
                <div
                  className={[
                    'max-w-[85%] rounded-2xl px-4 py-3 text-sm leading-relaxed',
                    isUser
                      ? 'bg-accent text-background'
                      : 'border border-white/[0.06] bg-white/[0.04] text-text/90',
                  ].join(' ')}
                >
                  {message.isBriefing && !isUser ? (
                    <p className="mb-2 text-[0.65rem] uppercase tracking-[0.14em] text-accent">
                      Daily briefing
                    </p>
                  ) : null}
                  <p className="whitespace-pre-wrap">{message.content}</p>
                </div>
              </div>
            )
          })}
          {sendResult.fetching ? (
            <div className="flex justify-start">
              <div className="rounded-2xl border border-white/[0.06] bg-white/[0.04] px-4 py-3 text-sm text-muted">
                <Loader2 className="size-4 animate-spin" />
              </div>
            </div>
          ) : null}
          <div ref={bottomRef} />
        </div>

        <form
          onSubmit={handleSubmit}
          className="shrink-0 border-t border-white/[0.06] p-3 sm:p-4"
        >
          <div className="flex gap-2">
            <input
              type="text"
              value={input}
              onChange={(event) => setInput(event.target.value)}
              placeholder="Ask about your holdings..."
              disabled={sendResult.fetching}
              className="min-w-0 flex-1 rounded-xl border border-white/[0.08] bg-background px-4 py-3 text-sm text-text placeholder:text-muted focus:border-accent/40 focus:outline-none disabled:opacity-50"
            />
            <button
              type="submit"
              disabled={!input.trim() || sendResult.fetching}
              className="shrink-0 rounded-xl bg-accent px-5 py-3 text-sm font-medium text-background hover:bg-accent/90 disabled:opacity-50"
            >
              Send
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}
