import { ApolloClient, gql, InMemoryCache } from "@apollo/client";
import { defer } from "rxjs";
import { map, shareReplay, switchMap, tap } from "rxjs/operators";

const anonClient = new ApolloClient({ cache: new InMemoryCache({}), uri: "/graphql" });

const query = gql`
	query AuthAnon {
		authAnon {
			uuid
		}
	}
`;

const localUuid$ = defer(() => [localStorage.getItem("sessionUuid")]);
const authUuid$ = defer(() => anonClient.query({ query })).pipe(
	map((res) => res.data.authAnon.uuid),
	tap((uuid) => localStorage.setItem("sessionUuid", uuid))
);
export const session$ = localUuid$.pipe(
	switchMap((res) => (res ? [res] : authUuid$)),
	shareReplay(1)
);
