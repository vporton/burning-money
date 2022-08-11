import { useEffect, useState } from 'react';
import { backendUrlPrefix } from './config';
import SumsubWebSdk from '@sumsub/websdk-react'

export function Kyc() {
    const [accessToken, setAccessToken] = useState("");

    async function updateAccessToken() {
        const accessToken = await (await fetch(`${backendUrlPrefix}/kyc/access-token`, {method: 'POST', credentials: 'include'})).text();
        console.log('accessToken:', accessToken);
        setAccessToken(accessToken);
    }

    useEffect(() => {
        updateAccessToken();
    }, []);

    return (accessToken !== "") ? <SumsubWebSdk
    accessToken={accessToken}
    updateAccessToken={updateAccessToken}
    expirationHandler={() => Promise.resolve(accessToken)}
    config={{
        lang: "ru-RU",
        email: "porton@narod.ru", // FIXME
        phone: "+972-545229306", // FIXME
        i18n: {
          document: {
            subTitles: {
              IDENTITY: "Upload a document that proves your identity"
            }
          }
        },
        onMessage: (type, payload) => {
          console.log("WebSDK onMessage", type, payload);
        },
        uiConf: {
          customCssStr:
            ":root {\n  --black: #000000;\n   --grey: #F5F5F5;\n  --grey-darker: #B2B2B2;\n  --border-color: #DBDBDB;\n}\n\np {\n  color: var(--black);\n  font-size: 16px;\n  line-height: 24px;\n}\n\nsection {\n  margin: 40px auto;\n}\n\ninput {\n  color: var(--black);\n  font-weight: 600;\n  outline: none;\n}\n\nsection.content {\n  background-color: var(--grey);\n  color: var(--black);\n  padding: 40px 40px 16px;\n  box-shadow: none;\n  border-radius: 6px;\n}\n\nbutton.submit,\nbutton.back {\n  text-transform: capitalize;\n  border-radius: 6px;\n  height: 48px;\n  padding: 0 30px;\n  font-size: 16px;\n  background-image: none !important;\n  transform: none !important;\n  box-shadow: none !important;\n  transition: all 0.2s linear;\n}\n\nbutton.submit {\n  min-width: 132px;\n  background: none;\n  background-color: var(--black);\n}\n\n.round-icon {\n  background-color: var(--black) !important;\n  background-image: none !important;\n}"
        },
        onError: (error) => {
          console.error("WebSDK onError", error);
        }
      }}
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
