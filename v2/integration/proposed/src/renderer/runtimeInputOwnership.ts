// Generic host prerequisite. Client revisions originate only at input time.
export type Ticket = { epoch: number; revision: number; request: number }
export class RuntimeInputOwnership {
  private epoch = 0
  private revision = 0
  private request = 0
  private pending: Ticket | null = null
  private ready = true
  private consumed = new Set<string>()
  private modelRevision = -1
  draft = ''
  reset(draft: string): void {
    this.epoch++; this.revision = 0; this.request++; this.pending = null
    this.ready = true; this.consumed.clear(); this.modelRevision = -1; this.draft = draft
  }
  input(raw: unknown): Ticket {
    // Revoke before even checking the type/size. Never dedupe A/B/A or controls.
    this.revision++; this.request++; this.ready = false; this.consumed.clear()
    const ticket = { epoch: this.epoch, revision: this.revision, request: this.request }
    this.pending = ticket
    if (typeof raw === 'string') this.draft = raw
    return ticket
  }
  read(): Ticket | null {
    if (this.pending) return null
    const ticket = { epoch: this.epoch, revision: this.revision, request: ++this.request }
    this.pending = ticket
    return ticket
  }
  action(id: string): Ticket | null {
    if (!this.ready || this.pending || this.consumed.has(id)) return null
    this.consumed.add(id)
    return this.read()
  }
  current(t: Ticket): boolean {
    return t.epoch === this.epoch && t.revision === this.revision && t.request === this.request
  }
  finish(t: Ticket): boolean {
    if (!this.current(t)) return false
    if (this.pending === t) this.pending = null
    return true
  }
  accept(t: Ticket, root: { props?: Record<string, unknown> }): boolean {
    if (!this.current(t)) return false
    const p = root.props ?? {}
    if (typeof p.nativeModelRevision === 'number') {
      // Echo verifies a client revision; it never advances it.
      if (p.clientInputRevision !== t.revision && !(t.revision === 0 && p.clientInputRevision == null)) return false
      if (p.searchText !== this.draft || p.nativeModelRevision < this.modelRevision) return false
      this.modelRevision = p.nativeModelRevision
    }
    this.ready = true
    return true
  }
  canAct(): boolean { return this.ready && this.pending === null }
}
