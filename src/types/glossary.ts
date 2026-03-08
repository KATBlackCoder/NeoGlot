export interface GlossaryEntry {
  id: number
  project_id: number | null // null = glossaire global
  term: string
  translation: string
  note: string
  match_mode: 'exact' | 'contains' | 'regex'
}

export interface NewGlossaryEntry {
  project_id: number | null
  term: string
  translation: string
  note?: string
  match_mode: GlossaryEntry['match_mode']
}
