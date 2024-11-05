import { HoudiniClient } from '$houdini';

const url =
	'http://localhost:8080/chains/e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65/applications/3326df61687858af29d28b71270a6ba8c259d7fb3a666ddef25f30038648d78cf933246bda05dd51d20395557cc6efa1b7fad0d5d0027d371059fc33978674bbe476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65c10000000000000000000000';

export default new HoudiniClient({
	url
	// url: 'http://localhost:8080'

	// uncomment this to configure the network call (for things like authentication)
	// for more information, please visit here: https://www.houdinigraphql.com/guides/authentication
	// fetchParams({ session }) {
	//     return {
	//         headers: {
	//             Authentication: `Bearer ${session.token}`,
	//         }
	//     }
	// }
});
