// Worst-case excursion analysis
function factorFromN(n) {
  const x0 = n;
  let x = 3*n + 1;
  let v = 0;
  while (x % 2 === 0) { x /= 2; v++; }
  let V = 1;
  while (x % 4 !== 1) {
    x = (3*x + 1) / 2;
    V++;
  }
  return {v, V, F: x / x0};
}

const stats = {};
const BAD = [], GOOD = [];

const LIMIT = 2000000;
let totalSum = 0, totalCount = 0, geoSum = 0;

for (let n = 1; n <= LIMIT; n += 2) {
  if (n % 4 !== 1) continue;
  const {v, V, F} = factorFromN(n);
  totalSum += F; totalCount++;
  const main = Math.pow(3, V) / Math.pow(2, v+V-1);
  geoSum += Math.log(main);
  
  const key = v+','+V;
  if (!stats[key]) stats[key] = {count: 0, sumF: 0, maxF: 0, minF: 1e9, gt1: 0, lt1: 0};
  stats[key].count++;
  stats[key].sumF += F;
  stats[key].maxF = Math.max(stats[key].maxF, F);
  stats[key].minF = Math.min(stats[key].minF, F);
  if (F > 1) stats[key].gt1++;
  else stats[key].lt1++;
  if (F > 1) BAD.push({n, v, V, F});
  else GOOD.push({n, v, V, F});
}

console.log('=== (v,V) distribution (n <= '+LIMIT+') ===');
const keys = Object.keys(stats).sort((a,b) => {
  const [av, aV] = a.split(',').map(Number);
  const [bv, bV] = b.split(',').map(Number);
  return av - bv || aV - bV;
});
for (const key of keys) {
  const s = stats[key];
  const [v,V] = key.split(',').map(Number);
  const main = Math.pow(3, V) / Math.pow(2, v+V-1);
  console.log(
    '(v='+v+' V='+V+') count='+s.count+
    ' pct='+(100*s.count/totalCount).toFixed(2)+'%'+
    ' avgF='+(s.sumF/s.count).toFixed(6)+
    ' main='+main.toFixed(6)+
    ' max='+s.maxF.toFixed(4)+
    ' min='+s.minF.toFixed(4)+
    ' >1:'+(100*s.gt1/s.count).toFixed(1)+'%'+
    ' <1:'+(100*s.lt1/s.count).toFixed(1)+'%'
  );
}

console.log('\nWorst BAD excursions (F > 1):');
BAD.sort((a,b) => b.F - a.F);
for (let i = 0; i < Math.min(15, BAD.length); i++) {
  console.log('  n='+BAD[i].n+' v='+BAD[i].v+' V='+BAD[i].V+' F='+BAD[i].F.toFixed(6));
}

console.log('\nBest GOOD excursions:');
GOOD.sort((a,b) => a.F - b.F);
for (let i = 0; i < Math.min(10, GOOD.length); i++) {
  console.log('  n='+GOOD[i].n+' v='+GOOD[i].v+' V='+GOOD[i].V+' F='+GOOD[i].F.toFixed(6));
}

console.log('\nOverall: avg F='+(totalSum/totalCount).toFixed(6)+
  ' geo_mean_main='+Math.exp(geoSum/totalCount).toFixed(6)+
  ' n_samples='+totalCount);
