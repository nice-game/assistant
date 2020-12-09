export interface CardProps {
	children?: React.ReactNode;
	className?: string;
}
export const Card: React.FunctionComponent<CardProps> = (props) => {
	return <div className={`mdc-card ${props.className}`}>{props.children}</div>
};
