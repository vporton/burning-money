import { RampInstantSDK } from '@ramp-network/ramp-instant-sdk';

function doInitiatePayment(userAddress) {
    new RampInstantSDK({
        hostAppName: 'World Token Credit Card Mine',
        hostLogoUrl: 'https://yourdapp.com/yourlogo.png',
        //   swapAmount: '150000000000000000000', // 150 ETH in wei
        swapAsset: 'DOT',
        fiatCurrency: 'USD',
        userAddress: '0x0', // FIXME
        //   hostApiKey: // FIXME
    }).on('*', event => console.log(event)).show();
}

function initiatePayment() {
    const userAddress = document.getElementById('userAccount');
    doInitiatePayment(userAddress);
}

(document.getElementById('initiatePayment') as HTMLButtonElement).onclick = initiatePayment;

