import { useEffect, useState } from 'react';
import { backendUrlPrefix } from './config';
import SumsubWebSdk from '@sumsub/websdk-react'

export function Kyc() {
    const [accessToken, setAccessToken] = useState("");

    async function fetchAccessToken() {
        const accessToken = await (await fetch(`${backendUrlPrefix}/kyc/access-token`, {method: 'POST', credentials: 'include'})).text();
        console.log('accessToken:', accessToken);
        return accessToken;
    }

    async function updateAccessToken() {
        setAccessToken(await fetchAccessToken());
    }

    useEffect(() => {
        updateAccessToken();
    }, []);

    return (accessToken !== "") ? <SumsubWebSdk
        accessToken={accessToken}
        // updateAccessToken={updateAccessToken}
        expirationHandler={fetchAccessToken}
        config={{}}
        options={{ addViewportTag: false, adaptIframeHeight: true }}
        onMessage={(type, payload) => {
            console.log("onMessage", type, payload);
        }}
        onError={(data) => console.log("onError", data)}
    /> : <span/>;
}

// async function launchWebSdk(accessToken) {
//     let snsWebSdkInstance = snsWebSdk.init(
//             accessToken,
//             // token update callback, must return Promise
//             () => this.getNewAccessToken()
//         )
//         .withBaseUrl('https://api.sumsub.com')
//         .withConf({
//             lang: 'en', //language of WebSDK texts and comments (ISO 639-1 format)
//         })
//         .on('onError', (error) => {
//           console.log('onError', error)
//         })
//         .onMessage((type, payload) => {
//           console.log('onMessage', type, payload)
//         })
//         .build();

//     // you are ready to go:
//     // just launch the WebSDK by providing the container element for it
//     snsWebSdkInstance.launch('#sumsub-websdk-container')
// }
