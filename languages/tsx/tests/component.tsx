import { Button } from "./button";

/** A greeting. */
export function Greeting({ name }: { name: string }) {
  const label = name || "you";
  return <Button label={label}>{name}</Button>;
}

export class Panel extends Base<Props> {
  /** Renders the panel. */
  render() {
    return this.props.visible ? <Greeting name="x" /> : null;
  }
}

export const Card = (props: Props) => <div className="card">{props.title}</div>;
