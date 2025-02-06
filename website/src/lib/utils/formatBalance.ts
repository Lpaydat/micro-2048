type FormatBalanceInput = string | number | null | undefined;

export const formatBalance = (balance: FormatBalanceInput): string => {
	if (!balance) return '0.0000';

	const numBalance = typeof balance === 'string' ? parseFloat(balance) : balance;
	return numBalance.toFixed(6);
};
