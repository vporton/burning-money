import { useEffect, useState } from 'react';
import { backendUrlPrefix } from './config';
import SumsubWebSdk from '@sumsub/websdk-react'

export function Kyc() {
    const [accessToken, setAccessToken] = useState("");

    function updateAccessToken() {
        fetch(backendUrlPrefix + "/email", {credentials: 'include'})
            .then(u => u.json())
            .then(async u => {
                const accessToken = await (await fetch(`${backendUrlPrefix}/kyc/access-token?userId=${u.email}`)).text();
                setAccessToken(accessToken);
            });
    }

    useEffect(() => {
        updateAccessToken();
    }, []);

    return <SumsubWebSdk
        accessToken={accessToken}
        updateAccessToken={updateAccessToken}
      />
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
