import type { Messages } from "./en";

export const ptBR: Messages = {
  appName: "Farol",
  empty: "Nenhuma porta aberta. Quando um servidor subir, ele aparece aqui.",
  otherUser: "outro usuário",
  pinnedGroup: "Fixados",
  outsideGroup: "Fora de worktree",
  forgotten: "esquecido?",
  search: "Buscar porta, nome, worktree ou origem",
  noResults: (query: string) => `Nada encontrado para "${query}".`,
  portCount: (n: number) => (n === 1 ? "1 porta aberta" : `${n} portas abertas`),

  front: "front-end",
  back: "back-end",
  switchType: (type: string) => `Detectado como ${type}. Clique para trocar.`,
  openInBrowser: (port: number) => `Abrir localhost:${port} no navegador`,
  openTerminal: "Abrir um terminal na pasta do processo",
  noFolder: "Pasta desconhecida",
  opening: (port: number) => `Abrindo localhost:${port}`,
  openingTerminal: "Abrindo o terminal",
  pin: "Fixar",
  unpin: "Desafixar",

  origins: {
    "claude-code": "Claude Code",
    cursor: "Cursor",
    vscode: "VS Code",
    terminal: "terminal",
    system: "sistema",
    unknown: "desconhecido",
  },

  stopLabel: (port: number) => `Encerrar o processo da porta ${port}`,
  noPermission: "Sem permissão",
  confirmStop: (name: string) => `Encerrar ${name}?`,
  cancel: "Cancelar",
  force: "Forçar",
  stop: "Encerrar",
  didNotStop: "Não encerrou. Forçar?",
  portFreed: (port: number) => `Porta ${port} liberada`,
  stillRunning: (port: number) => `A porta ${port} continua em uso`,

  errors: {
    "not-found": "O processo não existe mais",
    "permission-denied": "Sem permissão",
    "no-terminal": "Nenhum terminal encontrado",
    "not-implemented": "Ainda não disponível neste sistema",
    other: "Algo deu errado",
  },
};
