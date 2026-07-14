from decimal import Decimal, getcontext
getcontext().prec = 80
ONE = Decimal("1")
EPS_W = Decimal("0.000000001")

def geometric_mean(values):
    prod = Decimal(1)
    for v in values: prod *= Decimal(v)
    return prod ** (Decimal(1) / Decimal(len(values)))

def winsorize(values, lo, hi): return [min(max(Decimal(v), Decimal(lo)), Decimal(hi)) for v in values]
def minmax_normalize(value, lo, hi):
    lo=Decimal(lo); hi=Decimal(hi); value=Decimal(value)
    if hi <= lo: raise ValueError("bad range")
    return min(max((value-lo)/(hi-lo), Decimal(0)), Decimal(1))

def gate(x, lo, hi):
    x=Decimal(x); lo=Decimal(lo); hi=Decimal(hi)
    if x <= lo: return Decimal(0)
    if x >= hi: return ONE
    t=(x-lo)/(hi-lo)
    return t*t*(Decimal(3)-Decimal(2)*t)

def settlement_score(w_epoch_1, w_epoch_2):
    return geometric_mean([max(Decimal(w_epoch_1), EPS_W), max(Decimal(w_epoch_2), EPS_W)])
