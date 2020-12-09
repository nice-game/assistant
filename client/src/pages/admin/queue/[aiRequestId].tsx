import { AiRequest } from "@/components/ai-request";
import { Grid } from "@/components/material/layout-grid";
import { Textfield } from "@/components/material/textfield";
import { gql, useMutation, useQuery } from "@apollo/client";
import { useRouter } from "next/router";
import React, { useState } from "react";

const AdminQueueItem: React.FC = () => {
	const router = useRouter();
	const aiRequestId = Number(router.query.aiRequestId);

	const { data } = useQuery(query, { variables: { aiRequestId: aiRequestId } });

	const [createReply, { loading: loadingReply }] = useMutation(createReplyMutation, {
		onCompleted: () => router.push("/admin/queue"),
	});

	const [reply, setReply] = useState("");

	const onKeyPress = (event: React.KeyboardEvent<HTMLInputElement>) => {
		if (event.key === "Enter") {
			createReply({ variables: { aiRequestId, text: reply } });
			setReply("");
		}
	};

	return (
		<Grid className="container h-100vh">
			<Grid.Cell span={12} className="h-100 d-flex flex-col">
				<h1>Reply to Request</h1>
				<div className="flex-grow-1 overflow-y-auto bg-gray4 p-3 mb-3">
					{data && (
						<>
							{data.aiRequest.history.map((msg, i) => (
								<AiRequest key={i} aiRequest={msg} />
							))}
							<AiRequest
								aiRequest={{
									query: data.aiRequest.query,
									reply: loadingReply ? data.aiRequest.reply : undefined,
								}}
							/>
						</>
					)}
				</div>
				<Textfield
					label="Enter reply"
					disabled={loadingReply}
					outlined
					value={loadingReply ? "" : reply}
					valueChange={(x) => setReply(x)}
					onKeyPress={onKeyPress}
				/>
			</Grid.Cell>
		</Grid>
	);
};
export default AdminQueueItem;

const query = gql`
	query AiRequest($aiRequestId: Int!) {
		aiRequest(aiRequestId: $aiRequestId) {
			id
			sessionId
			query
			queryCreated
			history {
				id
				query
				queryCreated
				reply
				replyCreated
			}
		}
	}
`;

const createReplyMutation = gql`
	mutation CreateAiReply($aiRequestId: Int!, $text: String!) {
		createAiReply(aiRequestId: $aiRequestId, text: $text) {
			id
		}
	}
`;
