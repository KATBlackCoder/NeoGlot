export interface Project {
  id: number
  name: string
  game_path: string
  work_path: string
  engine: 'rpgmv' | 'rpgmz' | 'rpgmxp' | 'rpgmvx' | 'rpgmvxa' | 'wolf' | 'bakin' | 'unknown'
  source_lang: string
  target_lang: string
  project_context: string
  status: 'created' | 'extracted' | 'translating' | 'done'
  created_at: string
}

export interface NewProject {
  name: string
  game_path: string
  work_path: string
  engine: Project['engine']
  source_lang: string
  target_lang: string
  project_context: string
}

export interface ProjectProgress {
  total: number
  done: number
}

export const ENGINE_LABELS: Record<string, string> = {
  rpgmz: 'RPG Maker MZ',
  rpgmv: 'RPG Maker MV',
  rpgmxp: 'RPG Maker XP',
  rpgmvx: 'RPG Maker VX',
  rpgmvxa: 'RPG Maker VXAce',
  wolf: 'Wolf RPG',
  bakin: 'RPG Bakin',
  unknown: 'Inconnu',
}

export const PROJECT_STATUS_LABELS: Record<Project['status'], string> = {
  created: 'Créé',
  extracted: 'Extrait',
  translating: 'En cours',
  done: 'Terminé',
}

export const PROJECT_STATUS_VARIANTS: Record<
  Project['status'],
  'default' | 'secondary' | 'outline' | 'destructive'
> = {
  created: 'secondary',
  extracted: 'outline',
  translating: 'default',
  done: 'default',
}

export const STRING_STATUS_LABELS: Record<string, string> = {
  pending: 'En attente',
  translated: 'Traduit',
  reviewed: 'Validé',
}

export const STRING_STATUS_VARIANTS: Record<string, 'default' | 'secondary' | 'outline'> = {
  pending: 'outline',
  translated: 'secondary',
  reviewed: 'default',
}
