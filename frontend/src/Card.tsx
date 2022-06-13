import { RampInstantSDK } from '@ramp-network/ramp-instant-sdk';

export default function Card() {
    return <>
        <p>You mine World Token by using a credit card or a bank account (unlike Bitcoin that is mined by costly equipment).</p>
        <p>To mine an amount of World Token corresponding to a certain amount of money, pay any amount of money
            to your account <input type="text" id="userAccount"/>
            by <button onClick={initiatePayment}>clicking this button</button>
            first your account will be anonymously stored in our database and then you pay.
            After you paid, our system will initiate money transfer to your account.
            <strong>You must pay during 12 hours since clicking the button. If you pay late,
                your payment may be stalled in a crypto account with no one being able to get the money.</strong>
        </p>
    </>
}

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
