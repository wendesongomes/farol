import type { Messages } from "./en";

export const ptBR: Messages = {
  appName: "Farol",
  empty: "Nenhuma porta aberta. Quando um servidor subir, ele aparece aqui.",
  otherUser: "outro usuário",
  portCount: (n: number) => (n === 1 ? "1 porta aberta" : `${n} portas abertas`),
};
