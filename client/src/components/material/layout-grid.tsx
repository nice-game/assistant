import classNames from "classnames";

interface CellProps {
	children?: React.ReactNode;
	className?: string;
	span?: 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12;
}
const Cell: React.FunctionComponent<CellProps> = (props) => {
	const className = classNames(
		"mdc-layout-grid__cell",
		{ [`mdc-layout-grid__cell--span-${props.span}`]: props.span },
		props.className
	);

	return <div className={className}>{props.children}</div>;
};

interface GridProps {
	children?: React.ReactNode;
	className?: string;
}
const GridImpl: React.FunctionComponent<GridProps> = (props) => {
	return (
		<div className={`mdc-layout-grid ${props.className}`}>
			<div className="mdc-layout-grid__inner h-100">{props.children}</div>
		</div>
	);
};

export const Grid = Object.assign(GridImpl, { Cell });
