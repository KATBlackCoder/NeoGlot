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
  project_id: number
  total: number
  done: number
  percentage: number
}
