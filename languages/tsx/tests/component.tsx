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

export const element = <div />;

function Row(props: { id: string }) {
  return <li>{props.id}</li>;
}

const Item = (props: { id: string }) => <Row id={props.id} />;

export function List() {
  return (
    <ul>
      <Item id="a" />
      <Wrapper.Slot>
        <Row id="b" />
      </Wrapper.Slot>
    </ul>
  );
}
