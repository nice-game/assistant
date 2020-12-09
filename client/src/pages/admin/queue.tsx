import { gql, useQuery } from "@apollo/client";
import { format, parseJSON } from "date-fns";
import Link from "next/link";
import React from "react";

const AdminQueue: React.FC = () => {
	const { data } = useQuery(query);

	console.log(data?.unansweredAiRequests);
	return (
		<div className="container p-3">
			<h1>Request Queue</h1>
			{data &&
				data.unansweredAiRequests.map((aireq, i) => (
					<Link key={i} href={`/admin/queue/${aireq.id}`}>
						<a className="card mb-3">
							{aireq.query}
							<br />
							<small className="text-muted">{format(parseJSON(aireq.queryCreated), "P p")}</small>
						</a>
					</Link>
				))}
		</div>
	);
};
export default AdminQueue;

const query = gql`
	query UnansweredAiRequests {
		unansweredAiRequests {
			id
			sessionId
			query
			queryCreated
		}
	}
`;
