import type { Messages } from "./en";

export const ptBR: Messages = {
  appName: "Farol",
  empty: "Nenhuma porta aberta. Quando um servidor subir, ele aparece aqui.",
  otherUser: "outro usuário",
  portCount: (n: number) => (n === 1 ? "1 porta aberta" : `${n} portas abertas`),

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
