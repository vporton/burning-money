import { isAddress } from "ethers/lib/utils";

export default {
    isRealNumber: function(value: string) {
        return /^[0-9]+(\.[0-9]+)?$/.test(value)
    },
    isEthAddressValid: function(value: string) {
        return isAddress(value)
    },
}
