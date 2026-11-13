import { Button } from "./button";

export function Greeting({ name }) {
  const label = name || "you";
  return <Button label={label}>{name}</Button>;
}
